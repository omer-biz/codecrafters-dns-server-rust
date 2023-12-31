use std::{env::args, net::UdpSocket};

// The whole must be 12 bytes when encoded.
// which is 3 x 4 bytes.
#[derive(Default, PartialEq, Debug, Clone)]
struct Header {
    packet_id: u16, // 16 bit

    query_response_indicator: bool, // 1  bit
    operation_code: u8,             // 4  bits
    authoritative_answer: bool,     // 1  bit
    truncation: bool,               // 1  bit
    recursion_desired: bool,        // 1  bit

    recursion_available: bool, // 1  bit
    reserved: u8,              // 3  bits
    response_code: u8,         // 4  bits

    question_count: u16,          // 16 bits
    answer_record: u16,           // 16 bits
    authority_record_count: u16,  // 16 bits
    additional_record_count: u16, // 16 bits
}

impl Header {
    fn new(id: u16) -> Self {
        Self {
            packet_id: id,
            query_response_indicator: true,
            ..Self::default()
        }
    }

    fn query_response_indicator(self, query_response_indicator: bool) -> Self {
        Self {
            query_response_indicator,
            ..self
        }
    }

    fn operation_code(self, operation_code: u8) -> Self {
        Self {
            operation_code,
            ..self
        }
    }

    fn authoritative_answer(self, authoritative_answer: bool) -> Self {
        Self {
            authoritative_answer,
            ..self
        }
    }

    fn truncation(self, truncation: bool) -> Self {
        Self { truncation, ..self }
    }

    fn recursion_desired(self, recursion_desired: bool) -> Self {
        Self {
            recursion_desired,
            ..self
        }
    }

    fn recursion_available(self, recursion_available: bool) -> Self {
        Self {
            recursion_available,
            ..self
        }
    }

    fn reserved(self, reserved: u8) -> Self {
        Self { reserved, ..self }
    }

    fn response_code(self, response_code: u8) -> Self {
        Self {
            response_code,
            ..self
        }
    }
    fn question_count(self, question_count: u16) -> Self {
        Self {
            question_count,
            ..self
        }
    }

    fn answer_record(self, answer_record: u16) -> Self {
        Self {
            answer_record,
            ..self
        }
    }

    fn decode(buf: &[u8]) -> Self {
        let packet_id: u16 = (buf[0] as u16) << 8 | (buf[1] as u16);

        let query_response_indicator = /*-*/  (buf[2] & 0b10000000) == 0b10000000;
        let operation_code =           /*-*/  (buf[2] & 0b01111000) >> 3;
        let authoritative_answer =     /*-*/  (buf[2] & 0b00000100) == 0b00000100;
        let truncation =               /*-*/  (buf[2] & 0b00000010) == 0b00000010;
        let recursion_desired =        /*-*/  (buf[2] & 0b00000001) == 0b00000001;

        let recursion_available =      /*-*/  (buf[3] & 0b10000000) == 0b10000000;
        let reserved =                 /*-*/  (buf[3] & 0b01110000) >> 4;
        let response_code =            /*-*/   buf[3] & 0b00001111;

        let question_count: u16 = (buf[4] as u16) << 8 | (buf[5] as u16);
        let answer_record: u16 = (buf[6] as u16) << 8 | (buf[7] as u16);
        let authority_record_count: u16 = (buf[8] as u16) << 8 | (buf[9] as u16);
        let additional_record_count: u16 = (buf[10] as u16) << 8 | (buf[11] as u16);

        Self {
            packet_id,
            query_response_indicator,
            operation_code,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            reserved,
            response_code,
            question_count,
            answer_record,
            authority_record_count,
            additional_record_count,
        }
    }

    fn encode(&self) -> [u8; 12] {
        let part_two: u8 = (self.query_response_indicator as u8) << 7
            | (self.operation_code << 3)
            | (self.authoritative_answer as u8) << 2
            | (self.truncation as u8) << 1
            | (self.recursion_desired as u8);

        let part_three: u8 =
            (self.recursion_available as u8) << 7 | (self.reserved as u8) << 4 | self.response_code;

        [
            (self.packet_id >> 8) as u8,
            self.packet_id as u8,
            part_two,
            part_three,
            (self.question_count >> 8) as u8,
            self.question_count as u8,
            (self.answer_record >> 8) as u8,
            self.answer_record as u8,
            (self.authority_record_count >> 8) as u8,
            self.authority_record_count as u8,
            (self.additional_record_count >> 8) as u8,
            self.additional_record_count as u8,
        ]
    }
}

#[derive(Default, Debug, Clone)]
struct Question {
    name: Vec<u8>,
    type_: u16,
    class: u16,
}

impl Question {
    fn encode(&self) -> Vec<u8> {
        let mut question_encoded = vec![];

        question_encoded.extend(self.name.iter());
        question_encoded.extend(self.type_.to_be_bytes());
        question_encoded.extend(self.class.to_be_bytes());

        question_encoded
    }

    fn size(&self) -> usize {
        self.name.len() + 2 + 2
    }

    fn decode(buf: &[u8], offset: u16) -> Self {
        let mut size = offset as usize;
        let mut name = vec![];

        while buf[size] != 0x00 {
            if buf[size] & 0xc0 == 0xc0 {
                let offset =
                    ((buf[size] as u16) << 8 | (buf[size + 1] as u16)) & 0b0011111111111111;

                if offset > 1024 {
                    panic!("out of bounds")
                }

                size = offset as usize;
                // if there is a pointer this algorithm will
                // poplute the current questions type and
                // class with the pointer's type and class.
                //
                // btw I'm still confused by big endian and little endian.
                // I know what they are but still, confused.
            }
            name.push(buf[size]);
            size += 1;
        }

        name.push(0u8);
        size += 1; // consume the null byte

        let type_: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);
        size += 2;
        let class: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);

        Question { name, type_, class }
    }
}

#[derive(Debug, Clone)]
enum AnswerData {
    ARecord(u32),
}

impl Default for AnswerData {
    fn default() -> Self {
        AnswerData::ARecord(0u32)
    }
}

impl AnswerData {
    fn encode(&self) -> Vec<u8> {
        match self {
            AnswerData::ARecord(val) => val.to_be_bytes().to_vec(),
        }
    }
}

#[derive(Default, Debug, Clone)]
struct Answer {
    name: Vec<u8>,
    type_: u16,
    class: u16,
    ttl: u32,
    length: u16,
    data: AnswerData,
}

impl Answer {
    fn decode(buf: &[u8], offset: u16) -> Self {
        let mut size = offset as usize;
        let mut name = vec![];

        while buf[size] != 0x00 {
            if buf[size] & 0xc0 == 0xc0 {
                let offset =
                    ((buf[size] as u16) << 8 | (buf[size + 1] as u16)) & 0b0011111111111111;

                if offset > 1024 {
                    panic!("out of bounds")
                }

                size = offset as usize;
                // if there is a pointer this algorithm will
                // poplute the current questions type and
                // class with the pointer's type and class.
                //
                // btw I'm still confused by big endian and little endian.
                // I know what they are but still, confused.
            }
            name.push(buf[size]);
            size += 1;
        }

        name.push(0u8);
        size = name.len() + (offset as usize);

        let type_: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);
        size += 2;
        let class: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);
        size += 2;

        let ttl: u32 = (buf[size] as u32) << 24
            | (buf[size + 1] as u32) << 16
            | (buf[size + 2] as u32) << 8
            | (buf[size + 3]) as u32;
        size += 4;

        let length: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);
        size += 2;

        let ip: u32 = (buf[size] as u32) << 24
            | (buf[size + 1] as u32) << 16
            | (buf[size + 2] as u32) << 8
            | (buf[size + 3]) as u32;

        let data = AnswerData::ARecord(ip);

        Answer {
            name,
            type_,
            class,
            ttl,
            length,
            data,
        }
    }

    fn size(&self) -> usize {
        self.name.len() + 2 + 2 + 4 + 2 + 4
    }

    fn encode(&self) -> Vec<u8> {
        let mut encoded = vec![];

        encoded.extend(self.name.iter());
        encoded.extend(self.type_.to_be_bytes());
        encoded.extend(self.class.to_be_bytes());
        encoded.extend(self.ttl.to_be_bytes());
        encoded.extend(self.length.to_be_bytes());
        encoded.extend(self.data.encode());

        encoded
    }
}

#[derive(Default, Clone, Debug)]
struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl Message {
    fn encode(&self) -> Vec<u8> {
        let mut encoded = vec![];

        encoded.extend(self.header.encode());
        encoded.extend(self.questions.iter().flat_map(|q| q.encode()));
        encoded.extend(self.answers.iter().flat_map(|a| a.encode()));

        encoded
    }

    fn with_header(self, header: Header) -> Self {
        Self { header, ..self }
    }

    fn with_questions(self, questions: Vec<Question>) -> Self {
        Self {
            header: self.header.question_count(questions.len() as u16),
            questions,
            ..self
        }
    }

    fn with_answers(self, answers: Vec<Answer>) -> Self {
        Self {
            header: self.header.answer_record(answers.len() as u16),
            answers,
            ..self
        }
    }
}

fn decode_questions(nofq: u16, buf: &[u8]) -> Vec<Question> {
    let mut questions = vec![];
    let mut question_offset = 12;

    for _ in 0..nofq {
        let q = Question::decode(buf, question_offset);
        question_offset += q.size() as u16;

        questions.push(q);
    }

    questions
}

fn decode_answers(nofa: u16, buf: &[u8], offset: u16) -> Vec<Answer> {
    let mut answers = vec![];
    let mut answer_offset = offset;

    for _ in 0..nofa {
        let a = Answer::decode(buf, answer_offset);
        answer_offset += a.size() as u16;
        answers.push(a);
    }

    answers
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 1024];

    let resolver = args().nth(2).expect("Resolver not specified");
    let resolver_udp = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to address");

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_, source)) => {
                let header = Header::decode(&buf[..12]);
                let mut questions = vec![];

                if header.query_response_indicator == false && header.question_count > 0 {
                    questions = decode_questions(header.question_count, &buf);
                }

                let mut response_questions = vec![];
                let mut response_answers = vec![];
                for question in questions {
                    let mut innber_buf = [0; 1024];
                    let query = Message::default()
                        .with_header(header.clone())
                        .with_questions(vec![question]);

                    resolver_udp
                        .send_to(&query.encode(), &resolver)
                        .expect("Unable to send to resolver");

                    let (size, _) = resolver_udp
                        .recv_from(&mut innber_buf)
                        .expect("Failed to recieve from resolver");

                    let r_header = Header::decode(&innber_buf[..12]);

                    let questions = decode_questions(r_header.question_count, &innber_buf[..size]);
                    let offset: u16 = questions.iter().map(|q| q.size() as u16).sum();
                    response_questions.extend(questions);

                    let answers =
                        decode_answers(r_header.answer_record, &innber_buf[..size], offset + 12);

                    response_answers.extend(answers);
                }

                let mut header = Header::new(header.packet_id)
                    .query_response_indicator(true)
                    .authoritative_answer(false)
                    .truncation(false)
                    .recursion_available(false)
                    .recursion_desired(header.recursion_desired)
                    .operation_code(header.operation_code)
                    .truncation(header.truncation)
                    .reserved(0);

                if header.operation_code == 0 {
                    header = header.response_code(0);
                } else {
                    header = header.response_code(4);
                };

                let response = Message::default()
                    .with_header(header.clone())
                    .with_questions(response_questions)
                    .with_answers(response_answers);

                udp_socket
                    .send_to(&response.encode(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
