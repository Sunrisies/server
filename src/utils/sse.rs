use tokio::sync::broadcast;

// 全局事件广播器
#[derive(Clone)]
pub struct SseNotifier {
    tx: broadcast::Sender<String>,
}

impl SseNotifier {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        SseNotifier { tx }
    }
    // 创建一个新的事件通道
    pub fn create_channel(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
    // 发送消息
    pub fn notify(&self, msg: &str) {
        let _ = self.tx.send(msg.to_string());
    }
}

impl Default for SseNotifier {
    fn default() -> Self {
        Self::new()
    }
}
