use std::{fmt::Display, net::IpAddr, time::Duration};

const SPEED_MULTIPLE: usize = 1 << 10;

#[derive(Clone, Debug)]
pub enum Speed {
    Byte(f64),
    KByte(f64),
    MByte(f64),
    GByte(f64),
}

impl PartialEq for Speed{
    fn eq(&self, other: &Self) -> bool {
        let mut x = self.clone();
        let mut y = other.clone();
        x.byte();
        y.byte();
        match (x, y) {
            (Self::Byte(l0), Self::Byte(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl PartialOrd for Speed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut x = self.clone();
        let mut y = other.clone();
        x.byte();
        y.byte();
        match (x, y) {
            (Self::Byte(l0), Self::Byte(r0)) => {
                if l0 == r0 {
                    return Some(std::cmp::Ordering::Equal);
                }

                if l0 > r0 {
                    return Some(std::cmp::Ordering::Greater);
                }

                return Some(std::cmp::Ordering::Less);
            },
            _=> None
        }
    }
}

fn scale_up(x: &mut f64) {
    *x = *x * SPEED_MULTIPLE as f64
}

fn scale_down(x: &mut f64)  {
    *x = *x / SPEED_MULTIPLE as f64
}

impl Speed {
    pub fn byte_per_second(byte: usize, cost: Duration) -> Self {
        Speed::Byte(byte as f64 / cost.as_secs_f64())
    }

    fn unit_up(&mut self){
        match self {
            Speed::Byte(s)=> *self = Speed::KByte(*s),
            Speed::KByte(s)=> *self = Speed::MByte(*s),
            Speed::MByte(s)=> *self = Speed::GByte(*s),
            Speed::GByte(_)=> (),
        };
    }

    fn unit_down(&mut self){
        match self {
            Speed::GByte(s)=> *self = Speed::MByte(*s),
            Speed::MByte(s)=> *self = Speed::KByte(*s),
            Speed::KByte(s)=> *self = Speed::Byte(*s),
            Speed::Byte(_)=> (),
        };
    }

    fn upper(&mut self)  {
        match self {
            Speed::GByte(_) => (),
            Speed::Byte(s)|Speed::KByte(s)|Speed::MByte(s)=> {
                scale_down(s);
                self.unit_up();
            },
        }
    }
    
    fn lower(&mut self)  {
        match self {
            Speed::Byte(_)=>(),
            Speed::KByte(s)|Speed::MByte(s)|Speed::GByte(s)=> {
                scale_up(s);
                self.unit_down();
            },
        }
    }

    #[allow(dead_code)]
    pub fn byte(&mut self)  {
        match self {
            Speed::Byte(_) => (),
            Speed::KByte(_) => self.lower(),
            Speed::MByte(_) => {self.lower();self.lower()},
            Speed::GByte(_) => {self.lower();self.lower();self.lower()},
        };
    }



    #[allow(dead_code)]
    pub fn kb(&mut self)  {
        match self {
            Speed::Byte(_) => self.upper(),
            Speed::KByte(_) => (),
            Speed::MByte(_) => self.lower(),
            Speed::GByte(_) => {self.lower();self.lower()},
        }
    }

    pub fn mb(&mut self)  {
        match self {
            Speed::Byte(_) => {self.upper();self.upper()},
            Speed::KByte(_) => self.upper(),
            Speed::MByte(_) => (),
            Speed::GByte(_) => self.lower(),
        }
    }

    #[allow(dead_code)]
    pub fn gb(&mut self)  {
        match self {
            Speed::Byte(_) => {self.upper();self.upper();self.upper()},
            Speed::KByte(_) => {self.upper();self.upper()},
            Speed::MByte(_) => self.upper(),
            Speed::GByte(_) => (),
        }
    }
}

impl Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte(b) => write!(f, "{:.4} Byte/s", b),
            Self::KByte(b) => write!(f, "{:.4} KB/s", b),
            Self::MByte(b) => write!(f, "{:.4} MB/s", b),
            Self::GByte(b) => write!(f, "{:.4} GB/s", b),
        }
    }
}

pub struct DownloadTestStats {
    pub ip: IpAddr,
    pub speed: Speed,
}

impl Display for DownloadTestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<15} 下载速度 {}", self.ip, self.speed)
    }
}

impl DownloadTestStats {
    pub fn new(ip: IpAddr, speed: Speed) -> Self {
        Self { ip, speed }
    }
}

pub trait DownloadTest {
    fn download_test(&self) -> DownloadTestResult;
}

pub struct DownloadTestResult {
    pub top: usize,
    pub list: Option<Vec<DownloadTestStats>>,
}

impl Display for DownloadTestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.list {
            None => write!(f, "没有下载数据，可以尝试以下方法后再次重试：\n1. 更换网络\n2. 更新 Cloudflare 反代 IP 数据源\n3. 增大网络下载配置中 timeout 数值"),
            Some(list) => {
                let mut content = String::new();
                content.push_str("测速结果：\n");
                content.push_str(format!("总有效测速数据 {} 条\n", list.len()).as_str());
                content.push_str(format!("下面是下载速度最快的 {} 条数据：\n", self.top).as_str());
                list.iter().enumerate().all(|(i, x)| {
                    if i >= self.top {
                        return false
                    }
                    
                    content.push_str(format!("{}\n", x).as_str());
                    true
                });

                write!(f, "{}", content)
            }
        }
    }
}