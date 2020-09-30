use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tungstenite::{
    client::AutoStream, connect, error::Error as WebSocketError, stream::Stream as StreamSwitcher,
    Message, WebSocket,
};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OutputType {
    Nil,
    Verbose,
    Trace,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SCommandOption {
    audio_format: String,
    grammar_file_names: String,
    authorization: String,
}

impl SCommandOption {
    fn create_message(&self) -> String {
        format!(
            "s {} {} authorization={}",
            self.audio_format, self.grammar_file_names, self.authorization
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UEventToken {
    written: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UEventResult {
    tokens: Vec<UEventToken>,
    text: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct UEventPayload {
    results: Vec<UEventResult>,
    text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AEventToken {
    written: String,
    confidence: f32,
    starttime: u64,
    endtime: u64,
    spoken: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AEventResult {
    tokens: Vec<UEventToken>,
    confidence: f32,
    starttime: u64,
    endtime: u64,
    text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AEventPayload {
    results: Vec<AEventResult>,
    utteranceid: String,
    text: String,
    code: String,
    message: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
enum PacketData {
    SeSCommand(SCommandOption),
    SePCommand,
    SeECommand,
    ReSCommand(Option<String>),
    RePCommand(String),
    ReECommand(Option<String>),
    ReSEvent(u64),
    ReEEvent(u64),
    ReCEvent,
    ReUEvent(UEventPayload),
    ReAEvent(AEventPayload),
    ReGEvent(Option<String>),
    Others,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Packet {
    inserted_time: String,
    data: PacketData,
    raw: String,
}

impl Display for PacketData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PacketData::SeSCommand(_) => "Send s Command",
            PacketData::SePCommand => "Send p Command",
            PacketData::SeECommand => "Send e Command",
            PacketData::ReSCommand(_) => "Recieve s Command Response",
            PacketData::RePCommand(_) => "Recieve p Command Response",
            PacketData::ReECommand(_) => "Recieve e Command Response",
            PacketData::ReSEvent(_) => "Recieve S Event",
            PacketData::ReEEvent(_) => "Recieve E Event",
            PacketData::ReCEvent => "Recieve C Event",
            PacketData::ReUEvent(_) => "Recieve U Event",
            PacketData::ReAEvent(_) => "Recieve A Event",
            PacketData::ReGEvent(_) => "Recieve G Event",
            PacketData::Others => "Other Data",
        };
        write!(f, "{}", s)
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.inserted_time, self.data)
    }
}

trait MsgExt {
    fn get_packet(&self) -> Result<Option<Packet>, String>;
}

fn parse_command_error(packet_text: &String) -> Option<String> {
    if packet_text.len() > 2 {
        let message = &packet_text[2..];
        Some(message.to_string())
    } else {
        None
    }
}

fn get_timestamp() -> Result<String, String> {
    let now = SystemTime::now();
    let unixtime = now
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("failed to get unixtime: {}", e))?;
    Ok(format!(
        "{}.{:06}",
        unixtime.as_secs(),
        unixtime.subsec_micros()
    ))
}

fn convert_suffix_to_u64(suffix: Option<String>, message: &str) -> Result<u64, String> {
    let suffix = suffix.ok_or(message)?;
    suffix
        .parse()
        .map_err(|e| format!("failed to convert string to u64: {}", e))
}

trait DeserializePayload<T> {
    fn deserialize_packet(&self) -> Result<T, String>;
}

impl DeserializePayload<UEventPayload> for Option<String> {
    fn deserialize_packet(&self) -> Result<UEventPayload, String> {
        let suffix = self.as_ref().ok_or("failed to get U event payload")?;
        serde_json::from_str(suffix)
            .map_err(|e| format!("failed to deserialize U Event Payload: {}", e))
    }
}

impl DeserializePayload<AEventPayload> for Option<String> {
    fn deserialize_packet(&self) -> Result<AEventPayload, String> {
        let suffix = self.as_ref().ok_or("failed to A event payload")?;
        serde_json::from_str(suffix)
            .map_err(|e| format!("failed to deserialize A Event Payload: {}", e))
    }
}

impl MsgExt for Message {
    fn get_packet(&self) -> Result<Option<Packet>, String> {
        if let Message::Text(txt) = self {
            let command = &txt[..1];
            let suffix = parse_command_error(txt);
            let packet_data = match command {
                "s" => PacketData::ReSCommand(suffix),
                "p" => PacketData::RePCommand(suffix.ok_or("failed to get error message")?),
                "e" => PacketData::ReECommand(suffix),
                "S" => {
                    PacketData::ReSEvent(convert_suffix_to_u64(suffix, "failed to get start time")?)
                }
                "E" => {
                    PacketData::ReEEvent(convert_suffix_to_u64(suffix, "failed to get end time")?)
                }
                "C" => PacketData::ReCEvent,
                "U" => PacketData::ReUEvent(suffix.deserialize_packet()?),
                "A" => PacketData::ReAEvent(suffix.deserialize_packet()?),
                "G" => PacketData::ReGEvent(suffix),
                _ => PacketData::Others,
            };
            let packet = Packet {
                raw: txt.clone(),
                inserted_time: get_timestamp()?,
                data: packet_data,
            };
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonOutput {
    option: SCommandOption,
    packets: Vec<Packet>,
    lines: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

pub struct AmiWebSocketClient {
    output_data: JsonOutput,
    audio_file_reader: BufReader<File>,
    result_file_path: String,
    output_type: OutputType,
    socket: WebSocket<AutoStream>,
    is_end_initialize: bool,
    is_json_output: bool,
}

trait SendMessageExt<T> {
    fn send_message(&mut self, data: T) -> Result<bool, String>;
}

impl SendMessageExt<Vec<u8>> for AmiWebSocketClient {
    fn send_message(&mut self, data: Vec<u8>) -> Result<bool, String> {
        let packet = Packet {
            raw: "p<audio data>".to_string(),
            inserted_time: get_timestamp()?,
            data: PacketData::SePCommand,
        };

        match self.output_type {
            OutputType::Nil => (),
            OutputType::Verbose => println!("{}", packet),
            OutputType::Trace => println!("{:?}", packet),
        }

        let message = Message::Binary(data);

        if self.get_packets()? {
            return Ok(true);
        }
        self.socket
            .write_message(message)
            .map_err(|e| format!("failed to send message: {}", e))?;
        Ok(false)
    }
}

impl SendMessageExt<Packet> for AmiWebSocketClient {
    fn send_message(&mut self, data: Packet) -> Result<bool, String> {
        let message = Message::Text(data.raw.clone());

        match self.output_type {
            OutputType::Nil => (),
            OutputType::Verbose => println!("{}", data),
            OutputType::Trace => println!("{:?}", data),
        }

        if self.get_packets()? {
            return Ok(true);
        }
        self.socket
            .write_message(message)
            .map_err(|e| format!("failed to send message: {}", e))?;
        Ok(false)
    }
}

impl AmiWebSocketClient {
    pub fn new(
        api_key: String,
        audio_format: String,
        grammar_file_names: String,
        is_with_log: bool,
        is_json_output: bool,
        audio_file_path: String,
        result_file_path: &str,
        output_type: OutputType,
    ) -> Result<AmiWebSocketClient, String> {
        std::fs::write(&result_file_path, "")
            .map_err(|e| format!("failed to write result file (empty write for check): {}", e))?;

        let reader = File::open(audio_file_path)
            .map(|f| BufReader::new(f))
            .map_err(|e| format!("failed to open file: {}", e))?;
        let url = if is_with_log {
            Url::parse("wss://acp-api.amivoice.com/v1/").unwrap()
        } else {
            Url::parse("wss://acp-api.amivoice.com/v1/nolog/").unwrap()
        };
        let (mut socket, _) =
            connect(url).map_err(|e| format!("failed to connect by websocket: {}", e))?;
        let stream = socket.get_mut();
        let stream = match stream {
            StreamSwitcher::Plain(s) => s,
            StreamSwitcher::Tls(s) => s.get_mut(),
        };
        stream
            .set_nonblocking(true)
            .map_err(|e| format!("failed to set non blocking: {}", e))?;

        Ok(AmiWebSocketClient {
            output_data: JsonOutput {
                option: SCommandOption {
                    audio_format: audio_format.clone(),
                    grammar_file_names: grammar_file_names.clone(),
                    authorization: api_key.clone(),
                },
                packets: Vec::new(),
                lines: Vec::new(),
                error_message: None,
            },
            audio_file_reader: reader,
            output_type: output_type.clone(),
            result_file_path: result_file_path.to_string(),
            socket: socket,
            is_end_initialize: false,
            is_json_output: is_json_output,
        })
    }

    pub fn exec(&mut self) -> Result<(), String> {
        let result = self.exec_in_socket();
        if let Err(msg) = &result {
            self.output_data.error_message = Some(msg.clone());
            eprintln!("{}", msg);
        }

        let text = if self.is_json_output {
            serde_json::to_string_pretty(&self.output_data)
                .map_err(|e| format!("failed to serialize result: {}", e))?
        } else {
            self.output_data.lines.join("\n")
        };

        std::fs::write(&self.result_file_path, text)
            .map_err(|e| format!("failed to write result file: {}", e))?;

        self.socket
            .close(None)
            .map_err(|e| format!("failed to close websocket: {}", e))?;

        result
    }

    fn exec_in_socket(&mut self) -> Result<(), String> {
        self.start()?;
        while !self.is_end_initialize {
            sleep(Duration::from_millis(100));
            if self.get_packets()? {
                return Ok(());
            }
        }
        if self.send_audio()? {
            return Ok(());
        }

        loop {
            if self.get_packets()? {
                break;
            }
            sleep(Duration::from_micros(100));
        }

        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        let packet = Packet {
            raw: self.output_data.option.create_message(),
            inserted_time: get_timestamp()?,
            data: PacketData::SeSCommand(self.output_data.option.clone()),
        };
        self.send_message(packet).map(|_| ())
    }

    fn send_audio(&mut self) -> Result<bool, String> {
        let mut buf: [u8; 4096] = [0; 4096];

        loop {
            let index = self
                .audio_file_reader
                .read(&mut buf)
                .map_err(|e| format!("failed to read audio: {}", e))?;

            if index == 0 {
                break;
            }
            let mut binary = buf[..index].to_vec();
            binary.insert(0, 112);
            self.send_message(binary)?;

            // I do not know well. but need
            sleep(Duration::from_millis(5));
        }

        if self.get_packets()? {
            return Ok(true);
        }
        let packet = Packet {
            raw: "e".to_string(),
            inserted_time: get_timestamp()?,
            data: PacketData::SeECommand,
        };
        self.send_message(packet)?;

        Ok(false)
    }

    fn get_packets(&mut self) -> Result<bool, String> {
        while self.socket.can_read() {
            let msg = match self.socket.read_message() {
                Ok(msg) => msg,
                Err(e) => {
                    if let WebSocketError::Io(e) = e {
                        match e.kind() {
                            std::io::ErrorKind::WouldBlock => break,
                            _ => return Err(format!("failed to read message: {}", e)),
                        }
                    } else {
                        return Err(format!("failed to read message: {}", e));
                    }
                }
            };
            let packet = if let Some(packet) = msg.get_packet()? {
                packet
            } else {
                continue;
            };
            match self.output_type {
                OutputType::Nil => (),
                OutputType::Verbose => println!("{}", packet),
                OutputType::Trace => println!("{:?}", packet),
            }
            if self.is_json_output {
                self.output_data.packets.push(packet.clone());
            }
            match packet.data {
                PacketData::ReSCommand(msg) => {
                    self.is_end_initialize = true;
                    if let Some(msg) = msg {
                        return Err(msg);
                    }
                }
                PacketData::RePCommand(msg) => return Err(msg),
                PacketData::ReECommand(msg) => {
                    return if let Some(msg) = msg {
                        Err(msg)
                    } else {
                        Ok(true)
                    };
                }
                PacketData::ReAEvent(payload) => {
                    self.output_data.lines.push(payload.text.clone());
                }
                _ => (),
            }
        }
        Ok(false)
    }
}
