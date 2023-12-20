use std::net::UdpSocket;

// The whole must be 12 bytes when encoded.
// which is 3 x 4 bytes.
#[derive(Default, PartialEq, Debug)]
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

    fn packet_id(self, packet_id: u16) -> Self {
        Self { packet_id, ..self }
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

    fn authority_record_count(self, authority_record_count: u16) -> Self {
        Self {
            authority_record_count,
            ..self
        }
    }

    fn additional_record_count(self, additional_record_count: u16) -> Self {
        Self {
            additional_record_count,
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
        let response_code =            /*-*/  (buf[3] & 0b00001111);

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

#[derive(Default, Debug)]
struct Question {
    name: Vec<u8>,
    type_: u16,
    class: u16,
}

impl Question {
    fn new(name: &str) -> Self {
        Self {
            name: Self::encode_name(name),
            ..Self::default()
        }
    }

    fn encode_name(name: &str) -> Vec<u8> {
        let mut encoded_name = vec![];

        for label in name.split(".") {
            encoded_name.push(label.len() as u8);
            encoded_name.append(&mut label.as_bytes().to_owned());
        }

        encoded_name.push(0u8);

        encoded_name
    }

    fn with_type(self, type_: u16) -> Self {
        Self { type_, ..self }
    }

    fn with_class(self, class: u16) -> Self {
        Self { class, ..self }
    }

    fn with_name(self, name: &str) -> Self {
        Self {
            name: Self::encode_name(name),
            ..self
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut question_encoded = vec![];

        question_encoded.extend(self.name.to_owned());
        question_encoded.extend(self.type_.to_be_bytes());
        question_encoded.extend(self.class.to_be_bytes());

        question_encoded
    }

    fn size(&self) -> usize {
        self.name.len() + 2 + 2
    }

    fn decode(buf: &[u8]) -> Self {
        let mut size = 0;
        let mut name = vec![];

        while buf[size] != 0x00 {
            name.push(buf[size]);
            size += 1;
        }

        name.push(0u8);
        size += 1; // consume the null byte

        let type_: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);
        size += 2;
        let class: u16 = (buf[size] as u16) << 8 | (buf[size + 1] as u16);
        size += 2;

        Question { name, type_, class }
    }
}

#[derive(Debug)]
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

#[derive(Default, Debug)]
struct Answer {
    name: Vec<u8>,
    type_: u16,
    class: u16,
    ttl: u32,
    length: u16,
    data: AnswerData,
}

impl Answer {
    fn new(name: &str) -> Self {
        Self {
            name: Self::encode_name(name),
            ..Self::default()
        }
    }

    fn decode(buf: &[u8]) -> Self {
        let mut size = 0;
        let mut name = vec![];

        while buf[size] != 0x00 {
            name.push(buf[size]);
            size += 1;
        }

        name.push(0u8);
        size += 1; // consume the null byte

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

        let data = AnswerData::ARecord(0x08080808);
        size += 4;

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

    fn encode_name(name: &str) -> Vec<u8> {
        let mut encoded_name = vec![];

        for label in name.split(".") {
            encoded_name.push(label.len() as u8);
            encoded_name.append(&mut label.as_bytes().to_owned());
        }

        encoded_name.push(0u8);

        encoded_name
    }

    fn with_type(self, type_: u16) -> Self {
        Self { type_, ..self }
    }

    fn with_class(self, class: u16) -> Self {
        Self { class, ..self }
    }

    fn with_ttl(self, ttl: u32) -> Self {
        Self { ttl, ..self }
    }

    fn with_length(self, length: u16) -> Self {
        Self { length, ..self }
    }

    fn with_arcord(self, data: u32) -> Self {
        let data = AnswerData::ARecord(data);
        Self { data, ..self }
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

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut header = Header::decode(&buf[..12])
                    .query_response_indicator(true)
                    .authoritative_answer(false)
                    .truncation(false)
                    .recursion_available(false)
                    .reserved(0);
                header.question_count = 1;
                header.answer_record = 1;

                if header.operation_code == 0 {
                    header = header.response_code(0);
                } else {
                    header = header.response_code(4);
                }

                let question = Question::decode(&buf[12..]).with_type(1).with_class(1);
                let answer = Answer::decode(&buf[12..])
                    .with_type(1)
                    .with_class(1)
                    .with_ttl(60)
                    .with_length(4);

                let mut response = vec![];

                response.extend(header.encode());
                response.extend(question.encode());
                response.extend(answer.encode());

                println!("Received {} bytes from {}", size, source);
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
