use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::Result;
use serde_json::json;
use base64;

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
        println!("Attempting to connect to OpenAI at: {}", url);
        
        let mut req = url.into_client_request()?;   
        req.headers_mut().insert("Authorization", format!("Bearer {api_key}").parse()?);
        req.headers_mut().insert("OpenAI-Beta", "realtime=v1".parse()?);

        println!("Connecting to OpenAI WebSocket...");
        let (ws, response) = connect_async(req).await?;
        println!("OpenAI WebSocket connected successfully. Response status: {:?}", response.status());
        
        let (mut write, mut read) = ws.split();
        
        // Wait for OpenAI session response
        if let Some(msg) = read.next().await {
            println!("OpenAI session response: {:?}", msg);
        } else {
            return Err(anyhow::anyhow!("No initial response from OpenAI"));
        }
        
        // Send proper session.update configuration message
        println!("Sending session.update configuration...");
        let config = json!({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "instructions": system_prompt,
                "voice": "alloy",
                "input_audio_format": "pcm16",
                "output_audio_format": "pcm16",
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "prefix_padding_ms": 300,
                    "silence_duration_ms": 200,
                    "create_response": true,
                    "interrupt_response": true
                }
            }
        });
        write.send(Message::Text(config.to_string().into())).await?;

        println!("OpenAI connection setup complete");
        Ok(Self { write, read })
     }

    pub async fn send_audio(&mut self, data: axum::body::Bytes) -> Result<()>{
        // Convert audio data to base64 for OpenAI realtime API
        let audio_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
        
        // Create proper input_audio_buffer.append event
        let audio_event = json!({
            "type": "input_audio_buffer.append",
            "audio": audio_base64
        });
        
        self.write.send(Message::Text(audio_event.to_string().into())).await?;
        Ok(())
    }

    pub async fn commit_audio_buffer(&mut self) -> Result<()> {
        let commit_event = json!({
            "type": "input_audio_buffer.commit"
        });
        eprintln!("Sending input_audio_buffer.commit event");
        self.write.send(Message::Text(commit_event.to_string().into())).await?;
        eprintln!("input_audio_buffer.commit event sent successfully");
        Ok(())
    }

    pub async fn create_response(&mut self) -> Result<()> {
        let response_event = json!({
            "type": "response.create",
            "response": {
                "modalities": ["text", "audio"],
                "instructions": "Follow the Feynman tutor protocol."
            }
        });
        eprintln!("Sending response.create event");
        self.write.send(Message::Text(response_event.to_string().into())).await?;
        eprintln!("response.create event sent successfully");
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