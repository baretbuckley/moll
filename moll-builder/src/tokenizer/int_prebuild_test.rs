

static NUMBER: u32 = 1;

pub fn number(input: &str, l: usize) -> (usize, u32) {
    match input.chars().next() {
        Some(c) => {
            match c {
                '0'..='9' => number(&input[1..], l+1),
                '.' => number_dot(&input[1..], l+1),
                _ => (l, NUMBER)
            }
        },
        None => {(l, NUMBER)}
    }
}

pub fn number_dot(input: &str, l: usize) -> (usize, u32) {
    match input.chars().next() {
        Some(c) => {
            match c {
                '0'..='9' => number_tail(&input[1..], l+1),
                _ => (l, 0)
            }
        },
        None => {(l, 0)}
    }
}

pub fn number_tail(input: &str, l: usize) -> (usize, u32) {
    match input.chars().next() {
        Some(c) => {
            match c {
                '0'..='9' => number_tail(&input[1..], l+1),
                _ => (l, NUMBER)
            }
        },
        None => {(l, NUMBER)}
    }
}

/* 
 * Desired Equivilant
 * Tokenizer (
 *      Number {
 *          let $Digit > Self;
 *          let . > NumberDot;
 *          else > Terminate Number
 *      }
 *      NumberDot {
 *          let $Digit > NumberTail;
 *          else > Reject
 *      }
 *      NumberTail {
 *          let $Digit > Self;
 *          else > Terminate Number
 *      }
 * )
 * 
 */