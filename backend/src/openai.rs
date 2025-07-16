use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::Result;
use base64::Engine;

pub struct OASocket{
    // create a websocket object to send messages to OpenAI
    write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
    //create a websocket object to read messages from OpenAI
    read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    >,
}

impl OASocket{
    pub async fn connect(api_key: &str, system_prompt: &str) -> Result<Self>{
        let url = "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17";
        let mut req = url.into_client_request()?;   
        req.headers_mut().insert("Authorization", format!("Bearer {api_key}").parse()?);
        req.headers_mut().insert("OpenAI-Beta", "realtime=v1".parse()?);

        println!("Attempting to connect to OpenAI realtime API...");
        let (ws, response) = connect_async(req).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to OpenAI: {}", e))?;
        
        println!("Connected to OpenAI (status: {})", response.status());
        let (mut write, read) = ws.split();

        // Send session configuration according to OpenAI Realtime API spec
        let session_config = serde_json::json!({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "instructions": system_prompt,
                "voice": "alloy",
                "input_audio_format": "pcm16",
                "output_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": "whisper-1"
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "prefix_padding_ms": 300,
                    "silence_duration_ms": 200
                }
            }
        });

        println!("Sending session configuration to OpenAI...");
        write.send(Message::Text(session_config.to_string().into())).await
            .map_err(|e| anyhow::anyhow!("Failed to send session config: {}", e))?;

        println!("OpenAI connection established successfully");
        Ok(Self { write, read })
     }

    pub async fn send_audio(&mut self, data: axum::body::Bytes) -> Result<()>{
        // Convert Float32Array to PCM16 format for OpenAI
        let audio_event = serde_json::json!({
            "type": "input_audio_buffer.append",
            "audio": base64::prelude::BASE64_STANDARD.encode(&data)
        });
        
        self.write.send(Message::Text(audio_event.to_string().into())).await?;
        Ok(())
    }
    pub async fn next(&mut self) -> Result<Message> {
        let msg = self.read.next().await.ok_or_else(|| anyhow::anyhow!("Failed to receive message"))??;
        Ok(msg)
     }
    pub async fn close(&mut self) -> anyhow::Result<()> {
    use futures_util::SinkExt;
    self.write.send(tokio_tungstenite::tungstenite::Message::Close(None)).await?;
    Ok(())
    }
}