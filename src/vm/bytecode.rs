use super::opcodes::Opcode;

enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // Array(Vec<Constant>),
}

/*
🤓🤓🤓🤓🤓
🤓   🤓  🤓
🤓🤓🤓🤓🤓 Geometry Dash!!!!!!!! :)
🤓       🤓 I love geometry dash game(playing) RoboTop Games is my favorite game creator i love him so much
🤓🤓🤓🤓🤓 I wish to be like Him when i Grow up let Him into your hearts and repent
                His son died for our sins
                He is the only way to heaven
                He is the only way to God the Fathe
                He is the only way to the Holy Spirit

*/

struct Function {
    opcodes: Vec<Opcode>,
}

struct Bytecode {
    consts: Vec<Constant>,
    functions: Vec<Function>,
}
