use core::net::{Ipv4Addr, Ipv6Addr};
use heapless::Vec;

const MAX_SEND_PAYLOAD_SIZE: usize = 256;
const MAX_NAME_VIEW_LABELS: usize = 8;
const A_CODE: u16 = 1;
const AAAA_CODE: u16 = 28;
const CNAME_CODE: u16 = 5;
const MX_CODE: u16 = 15;
const TXT_CODE: u16 = 16;
const IN_FLAG_VALUE: u16 = 1;
const MAX_QUESIONS: usize = 8;
const MAX_ANSWERS: usize = 8;

pub enum DnsAction {
    SendPacket {
        payload: [u8; MAX_SEND_PAYLOAD_SIZE],
        len: usize,
    },
    Ignore,
}

pub trait DnsServer {
    fn handle_message(&mut self, buffer: &[u8]) -> DnsAction;
}

#[derive(Debug, PartialEq, defmt::Format)]
pub(super) enum DnsMessageType {
    Query,
    Response,
}

#[derive(Debug, PartialEq, defmt::Format)]
pub(super) enum DnsMessageOption {
    Standard,
    Inverse,
    Status,
}

#[derive(Debug, PartialEq, defmt::Format)]
pub(super) enum DnsResponseCode {
    Ok,
    FormatError,
    ServerFailure,
    NameError,
    TypeError,
    PolicyError,
}

#[derive(Debug, Clone, PartialEq, defmt::Format)]
pub(super) struct NameView<'a> {
    labels: Vec<&'a str, MAX_NAME_VIEW_LABELS>,
}

impl<'a> NameView<'a> {
    const TERMINATOR: u8 = 0x00;
    const POINTER_MASK: u8 = 0xC0;
    const OFFSET_MASK: u8 = 0x3F;
    const POINTER_SHIFT: u8 = 8;

    pub fn from_bytes(buffer: &'a [u8], mut offset: usize) -> Option<(Self, usize)> {
        let mut labels = Vec::new();
        let mut name_end = None;
        loop {
            let byte = *buffer.get(offset)?;
            if byte == Self::TERMINATOR {
                return Some((Self { labels }, name_end.unwrap_or(offset + 1)));
            } else if byte & Self::POINTER_MASK != 0 {
                if offset + 2 > buffer.len() {
                    return None;
                }
                let target = (((byte & Self::OFFSET_MASK) as usize) << Self::POINTER_SHIFT)
                    | buffer[offset + 1] as usize;
                if name_end.is_none() {
                    name_end = Some(offset + 2);
                }
                offset = target;
            } else {
                let len = byte as usize;
                if offset + 1 + len > buffer.len() {
                    return None;
                }
                let label = unsafe {
                    core::str::from_utf8_unchecked(&buffer[offset + 1..offset + 1 + len])
                };
                labels.push(label).ok()?;
                offset += 1 + len;
            }
        }
    }

    pub fn emit(&self, data: &mut [u8], mut offset: usize) -> Result<usize, ()> {
        for label in &self.labels {
            if offset + 1 + label.len() > data.len() {
                return Err(());
            }
            data[offset] = label.len() as u8;
            data[offset + 1..offset + 1 + label.len()].copy_from_slice(label.as_bytes());
            offset += 1 + label.len();
        }
        if offset >= data.len() {
            return Err(());
        }
        data[offset] = Self::TERMINATOR;
        Ok(offset + 1)
    }
}

#[derive(Debug, PartialEq, defmt::Format)]
pub(super) enum DnsQuestionType {
    A,
    Aaaa,
    Cname,
    Mx,
    Txt,
    Other(u16),
}

impl<'a> DnsQuestionType {
    pub fn from_bytes(buffer: &'a [u8], offset: usize) -> Option<(Self, usize)> {
        if offset + 2 > buffer.len() {
            return None;
        }
        let question_type_id = u16::from_be_bytes([buffer[offset], buffer[offset + 1]]);
        #[rustfmt::skip]
        let q_type = match question_type_id {
            A_CODE        => Self::A,
            AAAA_CODE     => Self::Aaaa,
            CNAME_CODE    => Self::Cname,
            MX_CODE       => Self::Mx,
            TXT_CODE      => Self::Txt,
            o                   => Self::Other(o),
        };
        Some((q_type, offset + 2))
    }

    pub fn emit(&self, data: &mut [u8], offset: usize) -> Result<usize, ()> {
        if offset + 2 > data.len() {
            return Err(());
        }
        #[rustfmt::skip]
        let q_type = match self {
                &Self::A        => A_CODE,
                &Self::Aaaa     => AAAA_CODE,
                &Self::Cname    => CNAME_CODE,
                &Self::Mx       => MX_CODE,
                &Self::Txt      => TXT_CODE,
                &Self::Other(o) => o,
            };
        data[offset..offset + 2].copy_from_slice(&q_type.to_be_bytes());
        Ok(offset + 2)
    }
}

#[derive(Debug, PartialEq, defmt::Format)]
#[rustfmt::skip]
pub(super) struct DnsQuestion<'a> {
    pub name:           NameView<'a>,
    pub question_type:  DnsQuestionType,
}

impl<'a> DnsQuestion<'a> {
    pub fn from_bytes(buffer: &'a [u8], offset: usize) -> Option<(Self, usize)> {
        let (name, offset) = NameView::from_bytes(buffer, offset)?;
        let (question_type, offset) = DnsQuestionType::from_bytes(buffer, offset)?;

        Some((
            Self {
                name,
                question_type,
            },
            offset + 2,
        ))
    }

    pub fn emit(&self, data: &mut [u8], offset: usize) -> Result<usize, ()> {
        let mut offset = self.name.emit(data, offset)?;
        offset = self.question_type.emit(data, offset)?;
        data[offset..offset + 2].copy_from_slice(&IN_FLAG_VALUE.to_be_bytes()); // class IN
        Ok(offset + 2)
    }
}

#[derive(Debug, PartialEq, defmt::Format)]
#[rustfmt::skip]
pub(super) enum DnsRecord<'a> {
    A       { addr: Ipv4Addr },
    Aaaa    { addr: Ipv6Addr },
    Cname   { name: NameView<'a> },
    Mx      { priority: u16, name: NameView<'a> },
    Txt     { data: &'a [u8] },
}

impl<'a> DnsRecord<'a> {
    pub fn from_bytes(
        buffer: &'a [u8],
        offset: usize,
        record_type: u16,
        len: u16,
    ) -> Option<(Self, usize)> {
        let end = offset + len as usize;
        if end > buffer.len() {
            return None;
        }
        let record = match record_type {
            A_CODE => {
                if len != 4 {
                    return None;
                }
                DnsRecord::A {
                    addr: Ipv4Addr::new(
                        buffer[offset],
                        buffer[offset + 1],
                        buffer[offset + 2],
                        buffer[offset + 3],
                    ),
                }
            }
            AAAA_CODE => {
                if len != 16 {
                    return None;
                }
                let addr = Ipv6Addr::from(*<&[u8; 16]>::try_from(&buffer[offset..end]).ok()?);
                DnsRecord::Aaaa { addr }
            }
            CNAME_CODE => {
                let (name, _) = NameView::from_bytes(buffer, offset)?;
                DnsRecord::Cname { name }
            }
            MX_CODE => {
                if len < 3 {
                    return None;
                }
                let priority = u16::from_be_bytes([buffer[offset], buffer[offset + 1]]);
                let (name, _) = NameView::from_bytes(buffer, offset + 2)?;
                DnsRecord::Mx { priority, name }
            }
            TXT_CODE => DnsRecord::Txt {
                data: &buffer[offset..end],
            },
            _ => return None,
        };
        Some((record, end))
    }

    pub fn emit(&self, data: &mut [u8], mut offset: usize) -> Result<(usize, u16, u16), ()> {
        match self {
            DnsRecord::A { addr } => {
                if offset + 4 > data.len() {
                    return Err(());
                }
                data[offset..offset + 4].copy_from_slice(&addr.octets());
                Ok((offset + 4, 4, A_CODE))
            }
            DnsRecord::Aaaa { addr } => {
                if offset + 16 > data.len() {
                    return Err(());
                }
                data[offset..offset + 16].copy_from_slice(&addr.octets());
                Ok((offset + 16, 16, AAAA_CODE))
            }
            DnsRecord::Cname { name } => {
                let new_offset = name.emit(data, offset)?;
                Ok((new_offset, (new_offset - offset) as u16, CNAME_CODE))
            }
            DnsRecord::Mx { priority, name } => {
                if offset + 2 > data.len() {
                    return Err(());
                }
                data[offset..offset + 2].copy_from_slice(&priority.to_be_bytes());
                let start_offset = offset;
                offset = offset + 2;
                let new_offset = name.emit(data, offset)?;
                Ok((new_offset, (new_offset - start_offset) as u16, MX_CODE))
            }
            DnsRecord::Txt { data: txt } => {
                if offset + txt.len() > data.len() {
                    return Err(());
                }
                data[offset..offset + txt.len()].copy_from_slice(txt);
                Ok((offset + txt.len(), txt.len() as u16, TXT_CODE))
            }
        }
    }
}

#[derive(Debug, PartialEq, defmt::Format)]
#[rustfmt::skip]
pub(super) struct DnsAnswer<'a> {
    pub name:   NameView<'a>,
    pub ttl:    u32,
    pub record: DnsRecord<'a>,
}

impl<'a> DnsAnswer<'a> {
    pub fn from_bytes(buffer: &'a [u8], offset: usize) -> Option<(Self, usize)> {
        let (name, offset) = NameView::from_bytes(buffer, offset)?;
        if offset + 10 > buffer.len() {
            return None;
        }
        let record_type = u16::from_be_bytes([buffer[offset], buffer[offset + 1]]);
        let ttl = u32::from_be_bytes([
            buffer[offset + 4],
            buffer[offset + 5],
            buffer[offset + 6],
            buffer[offset + 7],
        ]);
        let len = u16::from_be_bytes([buffer[offset + 8], buffer[offset + 9]]);
        let offset = offset + 10;
        let (record, offset) = DnsRecord::from_bytes(buffer, offset, record_type, len)?;
        Some((Self { name, record, ttl }, offset))
    }

    pub fn emit(&self, data: &mut [u8], offset: usize) -> Result<usize, ()> {
        let offset = self.name.emit(data, offset)?;
        let header_start = offset;
        let record_start = offset + 10;
        let (record_end, len, record_type) = self.record.emit(data, record_start)?;
        data[header_start..header_start + 2].copy_from_slice(&record_type.to_be_bytes());
        data[header_start + 2..header_start + 4].copy_from_slice(&IN_FLAG_VALUE.to_be_bytes()); // class IN
        data[header_start + 4..header_start + 8].copy_from_slice(&self.ttl.to_be_bytes());
        data[header_start + 8..header_start + 10].copy_from_slice(&len.to_be_bytes());
        Ok(record_end)
    }
}

#[derive(Debug, PartialEq, defmt::Format)]
#[rustfmt::skip]
pub(super) struct DnsRepr<'a> {
    identification:         u16,
    message_type:           DnsMessageType,
    message_option:         DnsMessageOption,
    authorative:            bool,
    truncated:              bool,
    recursion_desired:      bool,
    recursion_available:    bool,
    response_code:          DnsResponseCode,
    question_record_count:  u16,
    answer_record_count:    u16,
    questions:              Vec<DnsQuestion<'a>, MAX_QUESIONS>,
    answers:                Vec<DnsAnswer<'a>, MAX_ANSWERS>,
}

impl<'a> DnsRepr<'a> {
    const HEADER_SIZE: usize = 12;
    const MESSAGE_TYPE_MASK: u16 = 0x8000;
    const MESSAGE_OPTION_MASK: u16 = 0x7800;
    const MESSAGE_OPTION_SHIFT: usize = 11;
    const MESSAGE_OPTION_STANDARD_CODE: u16 = 0;
    const MESSAGE_OPTION_INVERSE_CODE: u16 = 1;
    const MESSAGE_OPTION_STATUS_CODE: u16 = 2;
    const MESSAGE_AUTHORATIVE_MASK: u16 = 0x0400;
    const MESSAGE_TRUNCATED_MASK: u16 = 0x0200;
    const MESSAGE_RECURSION_DESIRED_MASK: u16 = 0x0100;
    const MESSAGE_RECURSION_AVAILABLE_MASK: u16 = 0x0080;
    const MESSAGE_RESPONSE_CODE_MASK: u16 = 0x000F;
    const MESSAGE_RESPONSE_CODE_OK: u16 = 0;
    const MESSAGE_RESPONSE_CODE_FORMAT_ERROR: u16 = 1;
    const MESSAGE_RESPONSE_CODE_SERVER_FAILURE: u16 = 2;
    const MESSAGE_RESPONSE_CODE_NAME_ERROR: u16 = 3;
    const MESSAGE_RESPONSE_CODE_TYPE_ERROR: u16 = 4;
    const MESSAGE_RESPONSE_CODE_POLICY_ERROR: u16 = 5;

    pub fn from_bytes(buffer: &'a [u8]) -> Result<Self, ()> {
        if buffer.len() < Self::HEADER_SIZE {
            return Err(());
        }
        let flags = u16::from_be_bytes([buffer[2], buffer[3]]);
        let question_record_count = u16::from_be_bytes([buffer[4], buffer[5]]);
        let answer_record_count = u16::from_be_bytes([buffer[6], buffer[7]]);
        let mut questions: Vec<DnsQuestion, MAX_QUESIONS> = Vec::new();
        let mut offset = Self::HEADER_SIZE;
        for _ in 0..question_record_count {
            let (question, next_offset) = DnsQuestion::from_bytes(buffer, offset).ok_or(())?;
            questions.push(question).map_err(|_| ())?;
            offset = next_offset;
        }
        let mut answers: Vec<DnsAnswer, MAX_ANSWERS> = Vec::new();
        for _ in 0..answer_record_count {
            let (answer, next_offset) = DnsAnswer::from_bytes(buffer, offset).ok_or(())?;
            answers.push(answer).map_err(|_| ())?;
            offset = next_offset;
        }
        Ok(Self {
            identification: u16::from_be_bytes([buffer[0], buffer[1]]),
            message_type: if (flags & Self::MESSAGE_TYPE_MASK) != 0 {
                DnsMessageType::Response
            } else {
                DnsMessageType::Query
            },
            message_option: {
                let val = (flags & Self::MESSAGE_OPTION_MASK) >> Self::MESSAGE_OPTION_SHIFT;
                #[rustfmt::skip]
                let option = match val {
                    Self::MESSAGE_OPTION_STANDARD_CODE  => DnsMessageOption::Standard,
                    Self::MESSAGE_OPTION_INVERSE_CODE   => DnsMessageOption::Inverse,
                    Self::MESSAGE_OPTION_STATUS_CODE    => DnsMessageOption::Status,
                    _ => return Err(()),
                };
                option
            },
            authorative: (flags & Self::MESSAGE_AUTHORATIVE_MASK) != 0,
            truncated: (flags & Self::MESSAGE_TRUNCATED_MASK) != 0,
            recursion_desired: (flags & Self::MESSAGE_RECURSION_DESIRED_MASK) != 0,
            recursion_available: (flags & Self::MESSAGE_RECURSION_AVAILABLE_MASK) != 0,
            response_code: {
                let val = flags & Self::MESSAGE_RESPONSE_CODE_MASK;
                match val {
                    Self::MESSAGE_RESPONSE_CODE_OK => DnsResponseCode::Ok,
                    Self::MESSAGE_RESPONSE_CODE_FORMAT_ERROR => DnsResponseCode::FormatError,
                    Self::MESSAGE_RESPONSE_CODE_SERVER_FAILURE => DnsResponseCode::ServerFailure,
                    Self::MESSAGE_RESPONSE_CODE_NAME_ERROR => DnsResponseCode::NameError,
                    Self::MESSAGE_RESPONSE_CODE_TYPE_ERROR => DnsResponseCode::TypeError,
                    Self::MESSAGE_RESPONSE_CODE_POLICY_ERROR => DnsResponseCode::PolicyError,
                    _ => return Err(()),
                }
            },
            question_record_count,
            answer_record_count,
            questions,
            answers,
        })
    }

    pub fn emit(&self, buffer: &mut [u8]) -> Result<usize, ()> {
        buffer[0..2].copy_from_slice(&self.identification.to_be_bytes());
        let mut flags = 0u16;
        if matches!(self.message_type, DnsMessageType::Response) {
            flags |= Self::MESSAGE_TYPE_MASK;
        }
        #[rustfmt::skip]
        let opt = match self.message_option {
            DnsMessageOption::Standard  => Self::MESSAGE_OPTION_STANDARD_CODE,
            DnsMessageOption::Inverse   => Self::MESSAGE_OPTION_INVERSE_CODE,
            DnsMessageOption::Status    => Self::MESSAGE_OPTION_STATUS_CODE,
        };
        flags |= opt << Self::MESSAGE_OPTION_SHIFT;
        if self.authorative {
            flags |= Self::MESSAGE_AUTHORATIVE_MASK
        };
        if self.truncated {
            flags |= Self::MESSAGE_TRUNCATED_MASK
        };
        if self.recursion_desired {
            flags |= Self::MESSAGE_RECURSION_DESIRED_MASK
        };
        if self.recursion_available {
            flags |= Self::MESSAGE_RECURSION_AVAILABLE_MASK
        };
        #[rustfmt::skip]
        match self.response_code {
            DnsResponseCode::Ok             => flags |= Self::MESSAGE_RESPONSE_CODE_OK,
            DnsResponseCode::FormatError    => flags |= Self::MESSAGE_RESPONSE_CODE_FORMAT_ERROR,
            DnsResponseCode::ServerFailure  => flags |= Self::MESSAGE_RESPONSE_CODE_SERVER_FAILURE,
            DnsResponseCode::NameError      => flags |= Self::MESSAGE_RESPONSE_CODE_NAME_ERROR,
            DnsResponseCode::TypeError      => flags |= Self::MESSAGE_RESPONSE_CODE_TYPE_ERROR,
            DnsResponseCode::PolicyError    => flags |= Self::MESSAGE_RESPONSE_CODE_POLICY_ERROR,
        };
        buffer[2..4].copy_from_slice(&flags.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.question_record_count.to_be_bytes());
        buffer[6..8].copy_from_slice(&self.answer_record_count.to_be_bytes());
        let mut offset = Self::HEADER_SIZE;
        for question in &self.questions {
            offset = question.emit(buffer, offset)?;
        }
        for answer in &self.answers {
            offset = answer.emit(buffer, offset)?;
        }
        Ok(offset)
    }
}

pub mod poison {

    use crate::config::SERVER_IP;

    use super::*;

    const MAX_RESPONSE_SIZE: usize = 256;

    pub struct PoisonedDnsServer {}

    impl DnsServer for PoisonedDnsServer {
        fn handle_message(&mut self, buffer: &[u8]) -> DnsAction {
            let message = match DnsRepr::from_bytes(&buffer) {
                Ok(m) => m,
                Err(_) => return DnsAction::Ignore,
            };
            let mut answers = Vec::new();
            let question_name = match message.questions.get(0) {
                Some(question) => question.name.clone(),
                None => return DnsAction::Ignore,
            };
            let _ = answers.push(DnsAnswer {
                name: question_name,
                ttl: 0,
                record: DnsRecord::A { addr: SERVER_IP },
            });
            #[rustfmt::skip]
            let response = DnsRepr {
                identification:         message.identification,
                message_type:           DnsMessageType::Response,
                message_option:         message.message_option,
                authorative:            true,
                truncated:              false,
                recursion_desired:      message.recursion_desired,
                recursion_available:    true,
                response_code:          DnsResponseCode::Ok,
                question_record_count:  message.question_record_count,
                answer_record_count:    1,
                questions:              message.questions,
                answers,
            };
            let mut payload = [0u8; MAX_RESPONSE_SIZE];
            let len = match response.emit(&mut payload) {
                Ok(l) => l,
                Err(_) => return DnsAction::Ignore,
            };
            DnsAction::SendPacket { payload, len }
        }
    }

    impl PoisonedDnsServer {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_poisoned_dns_server_returns_server_ip() {
            let mut server = PoisonedDnsServer::new();
            let bytes: [u8; 35] = [
                61, 216, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 7, 99, 97, 112, 116, 105, 118, 101, 5, 97,
                112, 112, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
            ];
            let response = server.handle_message(&bytes);
            assert!(!matches!(response, DnsAction::Ignore));
            if let DnsAction::SendPacket { payload, .. } = response {
                let result =
                    DnsRepr::from_bytes(&payload).expect("failed to create repr from bytes");
                let answer: &DnsAnswer = result.answers.get(0).expect("no answers in response");
                assert_eq!(answer.record, DnsRecord::A { addr: SERVER_IP });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name_view_encode_decode_should_be_equal() {
        let name_view = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let mut buffer = [0u8; 64];
        name_view.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            NameView::from_bytes(&buffer, 0).expect("failed to create name view from bytes");
        assert_eq!(result, name_view);
        assert_eq!(offset, 10);
    }

    #[test]
    fn test_question_type_a_encode_decode_should_be_equal() {
        let question_type = DnsQuestionType::A;
        let mut buffer = [0u8; 64];
        question_type.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) = DnsQuestionType::from_bytes(&buffer, 0)
            .expect("failed to create question type from bytes");
        assert_eq!(result, question_type);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_question_type_aaaa_encode_decode_should_be_equal() {
        let question_type = DnsQuestionType::Aaaa;
        let mut buffer = [0u8; 64];
        question_type.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) = DnsQuestionType::from_bytes(&buffer, 0)
            .expect("failed to create question type from bytes");
        assert_eq!(result, question_type);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_question_type_cname_encode_decode_should_be_equal() {
        let question_type = DnsQuestionType::Cname;
        let mut buffer = [0u8; 64];
        question_type.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) = DnsQuestionType::from_bytes(&buffer, 0)
            .expect("failed to create question type from bytes");
        assert_eq!(result, question_type);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_question_type_mx_encode_decode_should_be_equal() {
        let question_type = DnsQuestionType::Mx;
        let mut buffer = [0u8; 64];
        question_type.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) = DnsQuestionType::from_bytes(&buffer, 0)
            .expect("failed to create question type from bytes");
        assert_eq!(result, question_type);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_question_type_txt_encode_decode_should_be_equal() {
        let question_type = DnsQuestionType::Txt;
        let mut buffer = [0u8; 64];
        question_type.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) = DnsQuestionType::from_bytes(&buffer, 0)
            .expect("failed to create question type from bytes");
        assert_eq!(result, question_type);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_question_encode_decode_should_be_equal() {
        let name = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let question = DnsQuestion {
            name,
            question_type: DnsQuestionType::A,
        };
        let mut buffer = [0u8; 64];
        question.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsQuestion::from_bytes(&buffer, 0).expect("failed to create question from bytes");
        assert_eq!(result, question);
        assert_eq!(offset, 14);
    }

    #[test]
    fn test_record_a_encode_decode_should_be_equal() {
        let record = DnsRecord::A {
            addr: Ipv4Addr::new(255, 127, 63, 32),
        };
        let mut buffer = [0u8; 64];
        record.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsRecord::from_bytes(&buffer, 0, 1, 4).expect("failed to create record from bytes");
        assert_eq!(result, record);
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_record_aaaa_encode_decode_should_be_equal() {
        let record = DnsRecord::Aaaa {
            addr: Ipv6Addr::new(255, 127, 63, 31, 15, 7, 3, 1),
        };
        let mut buffer = [0u8; 64];
        record.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsRecord::from_bytes(&buffer, 0, 28, 16).expect("failed to create record from bytes");
        assert_eq!(result, record);
        assert_eq!(offset, 16);
    }

    #[test]
    fn test_record_cname_encode_decode_should_be_equal() {
        let name = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let record = DnsRecord::Cname { name };
        let mut buffer = [0u8; 64];
        record.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsRecord::from_bytes(&buffer, 0, 5, 10).expect("failed to create record from bytes");
        assert_eq!(result, record);
        assert_eq!(offset, 10);
    }

    #[test]
    fn test_record_mx_encode_decode_should_be_equal() {
        let name = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let record = DnsRecord::Mx {
            priority: 187,
            name,
        };
        let mut buffer = [0u8; 64];
        record.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsRecord::from_bytes(&buffer, 0, 15, 11).expect("failed to create record from bytes");
        assert_eq!(result, record);
        assert_eq!(offset, 11);
    }

    #[test]
    fn test_record_txt_encode_decode_should_be_equal() {
        let record = DnsRecord::Txt {
            data: b"hello world",
        };
        let mut buffer = [0u8; 64];
        record.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsRecord::from_bytes(&buffer, 0, 16, 11).expect("failed to create record from bytes");
        assert_eq!(result, record);
        assert_eq!(offset, 11);
    }

    #[test]
    fn test_answer_encode_decode_should_be_equal() {
        let name = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let record = DnsRecord::A {
            addr: Ipv4Addr::new(255, 127, 63, 32),
        };
        let answer = DnsAnswer {
            name,
            ttl: 30,
            record,
        };
        let mut buffer = [0u8; 64];
        answer.emit(&mut buffer, 0).expect("failed to emit");
        let (result, offset) =
            DnsAnswer::from_bytes(&buffer, 0).expect("failed to create answer from bytes");
        assert_eq!(answer, result);
        assert_eq!(offset, 24);
    }

    #[test]
    fn test_repr_encode_decode_should_be_equal() {
        let name = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let question = DnsQuestion {
            name,
            question_type: DnsQuestionType::A,
        };
        let a_name = NameView {
            labels: heapless::Vec::from_iter(["test", "com"]),
        };
        let record = DnsRecord::A {
            addr: Ipv4Addr::new(255, 127, 63, 32),
        };
        let answer = DnsAnswer {
            name: a_name,
            ttl: 30,
            record,
        };
        let mut questions: heapless::Vec<DnsQuestion, MAX_QUESIONS> = Vec::new();
        let _ = questions.push(question);
        let mut answers: heapless::Vec<DnsAnswer, MAX_ANSWERS> = Vec::new();
        let _ = answers.push(answer);
        let repr = DnsRepr {
            identification: 1234,
            message_type: DnsMessageType::Response,
            message_option: DnsMessageOption::Standard,
            authorative: false,
            truncated: false,
            recursion_desired: false,
            recursion_available: false,
            response_code: DnsResponseCode::Ok,
            question_record_count: 1,
            answer_record_count: 1,
            questions,
            answers,
        };
        let mut buffer = [0u8; 64];
        let offset = repr.emit(&mut buffer).expect("failed to emit");
        assert_eq!(offset, 50);
        let result = DnsRepr::from_bytes(&buffer).expect("failed to create repr from bytes");
        assert_eq!(result, repr);
    }

    #[test]
    fn test_repr_from_bytes() {
        let bytes: [u8; 35] = [
            61, 216, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 7, 99, 97, 112, 116, 105, 118, 101, 5, 97, 112,
            112, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ];
        let result = DnsRepr::from_bytes(&bytes).expect("failed to create repr from bytes");
        let mut labels = Vec::new();
        let _ = labels.push("captive");
        let _ = labels.push("apple");
        let _ = labels.push("com");
        let mut questions = Vec::new();
        let _ = questions.push(DnsQuestion {
            name: NameView { labels },
            question_type: DnsQuestionType::A,
        });
        assert_eq!(
            result,
            DnsRepr {
                identification: 15832,
                message_type: DnsMessageType::Query,
                message_option: DnsMessageOption::Standard,
                authorative: false,
                truncated: false,
                recursion_desired: true,
                recursion_available: false,
                response_code: DnsResponseCode::Ok,
                question_record_count: 1,
                answer_record_count: 0,
                questions,
                answers: Vec::new(),
            }
        );
    }
}

pub mod mdns {

    use super::*;

    const MAX_RESPONSE_SIZE: usize = 256;
    const TTL: u32 = 120;

    pub struct MdnsServer {}

    impl MdnsServer {
        pub fn new() -> Self {
            Self {}
        }

        pub fn handle_message(&mut self, buffer: &[u8], ip: Ipv4Addr) -> DnsAction {
            let message = match DnsRepr::from_bytes(&buffer) {
                Ok(m) => m,
                Err(_) => return DnsAction::Ignore,
            };
            if message.message_type != DnsMessageType::Query {
                return DnsAction::Ignore;
            }
            let mut answers = Vec::new();
            for question in message.questions.iter() {
                let name = question.name.clone();
                if question.question_type == DnsQuestionType::A
                    && is_own_name(&name)
                    && !message.answers.iter().any(|answer| {
                        is_own_name(&answer.name)
                            && answer.record == DnsRecord::A { addr: ip }
                            && answer.ttl >= TTL / 2
                    })
                {
                    let _ = answers.push(DnsAnswer {
                        name,
                        ttl: TTL,
                        record: DnsRecord::A { addr: ip },
                    });
                }
            }
            if answers.is_empty() {
                return DnsAction::Ignore;
            }
            #[rustfmt::skip]
            let response = DnsRepr {
                identification:         0,
                message_type:           DnsMessageType::Response,
                message_option:         message.message_option,
                authorative:            true,
                truncated:              false,
                recursion_desired:      false,
                recursion_available:    false,
                response_code:          DnsResponseCode::Ok,
                question_record_count:  0,
                answer_record_count:    answers.len() as u16,
                questions:              Vec::new(),
                answers,
            };
            let mut payload = [0u8; MAX_RESPONSE_SIZE];
            let len = match response.emit(&mut payload) {
                Ok(l) => l,
                Err(_) => return DnsAction::Ignore,
            };
            DnsAction::SendPacket { payload, len }
        }
    }

    fn is_own_name(name: &NameView) -> bool {
        name.labels.len() == 2
            && "cresco".eq_ignore_ascii_case(name.labels[0])
            && "local".eq_ignore_ascii_case(name.labels[1])
    }

    #[cfg(test)]
    mod test {
        use super::*;

        const TEST_IP: Ipv4Addr = Ipv4Addr::new(10, 100, 237, 173);

        /// A browser resolving `cresco.local` sends three questions in a single packet -
        /// HTTPS (65), AAAA and A - and compresses the name of the second and third into
        /// pointers back to the first.
        #[rustfmt::skip]
        const BROWSER_QUERY: [u8; 42] = [
            0x00, 0x00,                                     // identification
            0x00, 0x00,                                     // flags: query
            0x00, 0x03,                                     // three questions
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x06, b'c', b'r', b'e', b's', b'c', b'o',
            0x05, b'l', b'o', b'c', b'a', b'l',
            0x00,
            0x00, 0x41, 0x80, 0x01,                         // HTTPS, IN + QU bit
            0xC0, 0x0C, 0x00, 0x1C, 0x80, 0x01,             // -> cresco.local, AAAA
            0xC0, 0x0C, 0x00, 0x01, 0x80, 0x01,             // -> cresco.local, A
        ];

        fn query_for(name: &[&str], question_type: u16) -> ([u8; 64], usize) {
            let mut buffer = [0u8; 64];
            buffer[5] = 0x01;
            let mut offset = 12;
            for label in name {
                buffer[offset] = label.len() as u8;
                buffer[offset + 1..offset + 1 + label.len()].copy_from_slice(label.as_bytes());
                offset += 1 + label.len();
            }
            offset += 1;
            buffer[offset..offset + 2].copy_from_slice(&question_type.to_be_bytes());
            buffer[offset + 2..offset + 4].copy_from_slice(&IN_FLAG_VALUE.to_be_bytes());
            (buffer, offset + 4)
        }

        #[test]
        fn test_browser_query_should_be_answered_with_the_current_ip() {
            let mut server = MdnsServer::new();
            let response = server.handle_message(&BROWSER_QUERY, TEST_IP);
            let DnsAction::SendPacket { payload, .. } = response else {
                panic!("expected a response to cresco.local");
            };
            let result = DnsRepr::from_bytes(&payload).expect("failed to create repr from bytes");
            assert_eq!(result.message_type, DnsMessageType::Response);
            assert!(result.authorative);
            assert_eq!(result.identification, 0);
            assert_eq!(result.answer_record_count, 1);
            let answer: &DnsAnswer = result.answers.get(0).expect("no answers in response");
            assert_eq!(answer.record, DnsRecord::A { addr: TEST_IP });
            assert_eq!(answer.name.labels.as_slice(), ["cresco", "local"]);
        }

        #[test]
        fn test_response_should_have_a_non_zero_ttl() {
            let mut server = MdnsServer::new();
            let DnsAction::SendPacket { payload, .. } =
                server.handle_message(&BROWSER_QUERY, TEST_IP)
            else {
                panic!("expected a response to cresco.local");
            };
            let result = DnsRepr::from_bytes(&payload).expect("failed to create repr from bytes");
            let answer: &DnsAnswer = result.answers.get(0).expect("no answers in response");
            assert_ne!(answer.ttl, 0);
        }

        #[test]
        fn test_response_should_not_contain_questions() {
            let mut server = MdnsServer::new();
            let DnsAction::SendPacket { payload, .. } =
                server.handle_message(&BROWSER_QUERY, TEST_IP)
            else {
                panic!("expected a response to cresco.local");
            };
            let result = DnsRepr::from_bytes(&payload).expect("failed to create repr from bytes");
            assert_eq!(result.question_record_count, 0);
            assert!(result.questions.is_empty());
        }

        #[test]
        fn test_query_for_another_host_should_be_ignored() {
            let mut server = MdnsServer::new();
            #[rustfmt::skip]
            let bytes: [u8; 47] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x0B, 0x74, 0x65, 0x73, 0x74, 0x2D, 0x63, 0x72, 0x65, 0x73, 0x63, 0x6F,
                0x05, 0x6C, 0x6F, 0x63, 0x61, 0x6C, 0x00,
                0x00, 0x41, 0x80, 0x01,
                0xC0, 0x0C, 0x00, 0x1C, 0x80, 0x01,
                0xC0, 0x0C, 0x00, 0x01, 0x80, 0x01,
            ];
            let response = server.handle_message(&bytes, TEST_IP);
            assert!(matches!(response, DnsAction::Ignore));
        }

        #[test]
        fn test_query_for_a_subdomain_should_be_ignored() {
            let mut server = MdnsServer::new();
            let (bytes, len) = query_for(&["cresco", "local", "example"], A_CODE);
            let response = server.handle_message(&bytes[..len], TEST_IP);
            assert!(matches!(response, DnsAction::Ignore));
        }

        #[test]
        fn test_query_without_an_a_question_should_be_ignored() {
            let mut server = MdnsServer::new();
            let (bytes, len) = query_for(&["cresco", "local"], AAAA_CODE);
            let response = server.handle_message(&bytes[..len], TEST_IP);
            assert!(matches!(response, DnsAction::Ignore));
        }

        #[test]
        fn test_single_label_query_should_be_ignored() {
            let mut server = MdnsServer::new();
            let (bytes, len) = query_for(&["local"], A_CODE);
            let response = server.handle_message(&bytes[..len], TEST_IP);
            assert!(matches!(response, DnsAction::Ignore));
        }

        #[test]
        fn test_hostname_should_match_case_insensitively() {
            let mut server = MdnsServer::new();
            let (bytes, len) = query_for(&["CRESCO", "LOCAL"], A_CODE);
            let response = server.handle_message(&bytes[..len], TEST_IP);
            assert!(matches!(response, DnsAction::SendPacket { .. }));
        }

        #[test]
        fn test_garbage_should_be_ignored() {
            let mut server = MdnsServer::new();
            let response = server.handle_message(&[0x00, 0x01, 0x02], TEST_IP);
            assert!(matches!(response, DnsAction::Ignore));
        }
    }
}
