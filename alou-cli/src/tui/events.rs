//! 事件处理模块

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

/// 应用程序事件
#[derive(Debug, Clone)]
pub enum Event {
    /// 定时器事件
    Tick,
    /// 键盘事件
    Key(KeyEvent),
    /// 鼠标事件
    Mouse(MouseEvent),
    /// 窗口大小改变事件
    Resize(u16, u16),
}

/// 事件处理器
#[derive(Debug)]
pub struct EventHandler {
    /// 事件发送器
    sender: mpsc::Sender<Event>,
    /// 事件接收器
    receiver: mpsc::Receiver<Event>,
    /// 事件处理线程
    handler: Option<thread::JoinHandle<()>>,
}

impl EventHandler {
    /// 创建新的事件处理器
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    // 等待事件或超时
                    if event::poll(timeout).expect("无法轮询事件") {
                        match event::read().expect("无法读取事件") {
                            CrosstermEvent::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e)).expect("无法发送键盘事件");
                                }
                            }
                            CrosstermEvent::Mouse(e) => {
                                sender.send(Event::Mouse(e)).expect("无法发送鼠标事件");
                            }
                            CrosstermEvent::Resize(w, h) => {
                                sender.send(Event::Resize(w, h)).expect("无法发送调整大小事件");
                            }
                            _ => {}
                        }
                    }

                    // 发送定时器事件
                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("无法发送定时器事件");
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            sender,
            receiver,
            handler: Some(handler),
        }
    }

    /// 获取下一个事件
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        // 发送停止信号
        let _ = self.sender.send(Event::Key(KeyEvent::new(
            crossterm::event::KeyCode::Null,
            crossterm::event::KeyModifiers::NONE,
        )));
        
        // 等待线程结束 - 使用 take 避免 move 问题
        if let Some(handler) = self.handler.take() {
            let _ = handler.join();
        }
    }
}