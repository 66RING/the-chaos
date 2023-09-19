use std::net::{ToSocketAddrs, TcpListener, TcpStream};
use std::io::{BufReader, BufWriter, Write};
use serde_json::Deserializer;
use crate::common::{Result, Request, GetResponse};

pub struct Server {
}

impl Server {
    /// 创建server
    pub fn new() -> Self {
        Self {}
    }

    /// 启动server
    pub fn run<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        // 监听tcp端口
        // while loop不断接收请求然后serve请求
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(_) = self.serve(stream) {
                        panic!("error");
                    }
                }
                Err(_) => panic!("error"),
            }
        }
        Ok(())
    }

    /// 处理请求
    pub fn serve(&self, tcp: TcpStream) -> Result<()> {
        // 获取客户端地址
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        /// 发送序列化后的回复
        macro_rules! send_reap {
            ($resp:expr) => {{
                let resp = $resp;
                serde_json::to_writer(&mut writer, &resp)?;
                writer.flush()?;
                // println!("Response send to {}: {:?}", peer_addr, resp);
            }};
        }

        for req in req_reader {
            let req = req?;
            println!("Receive request from {}: {:?}", peer_addr, req);
            match req {
                Request::Echo(msg) => send_reap!(match self.echo(msg) {
                    Ok(value) => GetResponse::Ok(Some(value)),
                    Err(_) => GetResponse::Err(format!("resp err")),
                }),
                Request::Hello => send_reap!(match self.hello() {
                    Ok(_) => GetResponse::Ok(Some(format!("ok"))),
                    Err(_) => GetResponse::Err(format!("resp err")),
                }),
            }
        }
        Ok(())
    }

    fn echo(&self, msg: String) -> Result<String> {
        println!("req: {}", msg);
        Ok("echo Ok".to_string())
    }
    fn hello(&self) -> Result<String> {
        println!("req: hello");
        Ok("hello Ok".to_string())
    }
}
