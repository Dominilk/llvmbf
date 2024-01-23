use thiserror::Error;

#[derive(Error, PartialEq, Eq, Debug)]
pub enum ParseError {
    #[error("Unmatched jump at position {position}.")]
    UndelimitedJump {
        position: usize
    },
}

/// Enum denoting a brainf*ck instruction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Instruction {
    MoveLeft,
    MoveRight,

    Increment,
    Decrement,

    Output,
    Input,

    Loop(Vec<Instruction>),

    /// Noop instruction, used for unknown instructions to pad the instruction vector.
    Noop
}

/// Parses a string of brainf*ck code into a vector of instructions.
pub fn parse(index_offset: usize, code: &str) -> Result<Vec<Instruction>, ParseError> {
    let mut instructions = Vec::default();
    
    let mut index = 0;

    while let Some(character) = code.chars().nth(index) { // TODO: linearly iterate through chars instead of index mess.
        let instruction = match character {
            '<' => Instruction::MoveLeft,
            '>' => Instruction::MoveRight,
            '+' => Instruction::Increment,
            '-' => Instruction::Decrement,
            '.' => Instruction::Output,
            ',' => Instruction::Input,
            '[' => {
                let begin = index;
                let mut open = 1usize;
                
                while open > 0 {
                    index += 1;

                    match code.chars().nth(index) {
                        Some('[') => open += 1,
                        Some(']') => open -= 1,
                        None => return Err(ParseError::UndelimitedJump { position: index_offset + begin }),
                        _ => {},
                    }
                }

                let loop_content = parse(begin, &code[begin + 1..index])?;

                Instruction::Loop(loop_content)
            },
            ']' => return Err(ParseError::UndelimitedJump { position: index_offset + index }),
            _ => Instruction::Noop
        };

        instructions.push(instruction);

        index += 1;
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse(0, "ABC><+-.,[]").unwrap(), vec![
            Instruction::Noop,
            Instruction::Noop,
            Instruction::Noop,

            Instruction::MoveRight,
            Instruction::MoveLeft,
            Instruction::Increment,
            Instruction::Decrement,
            Instruction::Output,
            Instruction::Input,
            Instruction::Loop(Vec::default()),
        ]);
    }

    #[test]
    fn test_parse_unmatched_jump() {
        assert_eq!(parse(0, "["), Err(ParseError::UndelimitedJump { position: 0 }));

        assert_eq!(parse(0, "++]"), Err(ParseError::UndelimitedJump { position: 2 }));
    }
}