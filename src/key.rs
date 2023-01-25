//// Functions related to key.

/// Represents *key*.
pub type Key = char;
type Position = (u8, u8);

/// Represents *distance* between keys.
pub type Distance = u16;

/// Character to key.
///
/// # Examples
/// 
/// ````
/// assert_eq!(to_key('a'), 'A');
/// assert_eq!(to_key('Z'), 'Z');
/// ````
pub fn to_key(c: char) -> Key {
    c.to_ascii_uppercase()
}

/// Returns `(row, rel. position)` of the key `k`.
///
/// # Examples
///
/// ```
/// assert_eq!(position('Z'), (0, 0));
/// assert_eq!(position('H'), (1, 5));
/// ```
pub fn position(k: Key) -> Position {
    "ZXCVBNMASDFGHJKLQWERTYUIOP1234567890"
    .find(k)
    .and_then(|p| match p {
         26..=36 => Some((3, (p - 26) as u8)), //1234567890
         16..=25 => Some((2, (p - 16) as u8)),  //QWERTYUIOP
         7..=15  => Some((1, (p - 7)  as u8)),   //ASDFGHJKL
         _       => Some((0,  p       as u8)),    //ZXCVBKM
     })
    .expect(
         format!(
             r#"
                Oh no!
                Key should be given as a capital letter or digit!
                Given: '{k}' (not in 'A'..='Z' or '0'..='9').
             "#
         ).as_str()
    )
 }

/// Returns the distance between `k1` and `k2` keys.
///
/// ! Be aware that this is not how it works on a real keyboard :),
/// ! the distance from `A` to `Q`, `W`, `S`, `Z` is all `1`, not
/// ! `1`, `2`, `1`, `1`.
///
/// # Examples
///
/// ```
/// assert_eq!(distance('F', 'H'), 2);
/// assert_eq!(distance('H', 'F'), 2);
/// assert_eq!(distance('Z', 'Q'), 2);
/// assert_eq!(distance('A', 'L'), 8);
/// assert_eq!(distance('A', 'L'), 8);
/// ```
pub fn distance(k1: char, k2: char) -> Distance {
    let (r1, p1) = position(k1);
    let (r2, p2) = position(k2);

    (
       (r1 as i8 - r2 as i8).abs()
     + (p1 as i8 - p2 as i8).abs()
    ) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_f() {
        assert_eq!(position('Z'), (0, 0));
        assert_eq!(position('M'), (0, 6));
        assert_eq!(position('7'), (3, 6));
        assert_eq!(position('A'), (1, 0));
        assert_eq!(position('K'), (1, 7));
        assert_eq!(position('L'), (1, 8));

        assert!(std::panic::catch_unwind(|| position('k')).is_err());
    }

    #[test]
    fn distance_f() {
        assert_eq!(distance('F', 'H'), 2);
        assert_eq!(distance('H', 'F'), 2);
        assert_eq!(distance('Z', 'Q'), 2);
        assert_eq!(distance('A', 'L'), 8);
        assert_eq!(distance('A', 'L'), 8);
        assert_eq!(distance('Z', 'R'), 5);
        assert_eq!(distance('Z', '0'), 12);
    }
}
