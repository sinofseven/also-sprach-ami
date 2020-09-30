also-sprach-ami
===

日本語の音声認識で人気の[AmiVoice Cloud Platform](https://acp.amivoice.com/main/)をかんたんに使うための非公式CLIツールです。

ドキュメントは気が向いたら整備します。

## インストール方法
```
cargo install also-sprach-ami
```

## 使い方
### transcribe
```
USAGE:
    also-sprach-ami transcribe [FLAGS] [OPTIONS] --audio-path <audio_file> --output-file <output_file>

FLAGS:
    -h, --help              Prints help information
        --no-log            flag of saving audio file and recognition result
        --is-json-output    flag of output json
        --trace             
    -V, --version           Prints version information
    -v, --verbose           

OPTIONS:
        --api-key <api_key>                          AmiVoice Cloud Platform API KEY
        --audio-path <audio_file>                    target audio file path
        --audio-foramt <audio_format>
            audio file foramt. Details:
            https://acp.amivoice.com/main/manual/%e9%9f%b3%e5%a3%b0%e3%83%95%e3%82%a9%e3%83%bc%e3%83%9e%e3%83%83%e3%83%88%e3%81%ab%e3%81%a4%e3%81%84%e3%81%a6/
             [default: 16k]
        --grammar-file-names <grammar_file_names>    Types of Speech Recognition Engines [default: -a-general]
        --output-file <output_file>                  output file path
```

AmiVoice Cloud PlatformのWebSocket APIを使用して、音声から日本語を認識します。  
音声フォーマットや認識エンジンについてはAmiVoice Cloud Platformの方を確認してください。  

API KEYをローカルにも保存していない かつ 引数でも指定していない場合、対話的にAPI KEYの入力が求められます。

***注意***  
音声ファイルの大きさによっては、完了まで数分以上の時間がかかることがあります。  
ご注意ください。


### configure
```bash
also-sprach-ami configure
```


AmiVoice Cloud PlatformのAPI KEYをローカルに保存します。  
実行すると対話的に入力を求められます。
ローカルに保存することでtranscribeのたびにAPI KEYを入力しなくてもよくなります。

