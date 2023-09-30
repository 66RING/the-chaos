use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};
use crate::common::{Result, Request, GetResponse, MyError};

pub struct Client {
    // 消息都是根据协议序列化后发送的, reader需要反序列化
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    // 将消息序列化后写入writer
    writer: BufWriter<TcpStream>,
}

impl Client {
    /// 创建连接对象
    pub fn connect<T: ToSocketAddrs>(addr: T) -> Result<Self> {
        // 建立tcp连接
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;
        Ok(Self {
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)),
            writer: BufWriter::new(tcp_writer),
        })
    }

    pub fn echo(&mut self, msg: String) -> Result<()> {
        // 向server发送请求
        serde_json::to_writer(&mut self.writer, &Request::Echo(msg))?;
        self.writer.flush()?;
        // 获取response
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(ok) => {
                println!("{}", ok.unwrap());
                Ok(())
            },
            GetResponse::Err(e) => Err(MyError::Any(e)),
        }
    }
    pub fn hello(&mut self) -> Result<()> {
        // 向server发送请求
        serde_json::to_writer(&mut self.writer, &Request::Hello)?;
        self.writer.flush()?;
        // 获取response
        let resp = GetResponse::deserialize(&mut self.reader)?;
        match resp {
            GetResponse::Ok(ok) => {
                println!("{}", ok.unwrap());
                Ok(())
            },
            GetResponse::Err(e) => Err(MyError::Any(e)),
        }
    }
}
