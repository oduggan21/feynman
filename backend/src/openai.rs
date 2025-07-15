use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::Result;

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
        let url = "wss://api.openai.com/v1/realtime/ws/audio?model=gpt-4o-realtime-preview";
        let mut req = url.into_client_request()?;   
        req.headers_mut().insert("Authorization", format!("Bearer {api_key}").parse()?);
        req.headers_mut().insert("OpenAI-Beta", "realtime=v1".parse()?);

        let (ws, _) = connect_async(req).await?;
        let (mut write, read) = ws.split();

        write.send(Message::Text(
                r#"{"audio":{"sample_rate":48000,"channels":1,"voice":"alloy"}}"#.into(),
            ))
            .await?;

        write
            .send(Message::Text(
                format!(
                    r#"{{"messages":[{{"role":"system","content":"{system_prompt}"}}]}}"#,
                ).into()
            ))
            .await?;

        Ok(Self { write, read })
     }

    pub async fn send_audio(&mut self, data: axum::body::Bytes) -> Result<()>{
            self.write.send(Message::Binary(data)).await?;
            Ok(())
         }
    pub async fn next(&mut self) -> Result<Message> {
        let msg = self.read.next().await.ok_or_else(|| anyhow::anyhow!("Failed to receive message"))??;
        Ok(msg)
     }
}