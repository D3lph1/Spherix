use std::collections::HashMap;

pub fn stringed_float_to_i64(s: &str, precision: u32) -> anyhow::Result<i64> {
    // Split a string into integer and fractional parts
    let parts: Vec<&str> = s.split('.').collect();
    
    let negative = parts[0].starts_with("-");
    
    let integer_part = parts[0].parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Invalid integer part"))?;

    // Fractional part or empty string if it is missing
    let fractional_part = if parts.len() > 1 { parts[1] } else { "" };

    // Processing fractional part for specified accuracy
    let mut fractional_value = fractional_part.chars()
        .take(precision as usize)
        .collect::<String>();

    if fractional_value.len() == 0 {
        fractional_value = "0".to_owned();
    }

    // We add zeros if the fractional part is not enough to reach the required accuracy.
    while fractional_value.len() < precision as usize {
        fractional_value.push('0');
    }

    // Convert the fractional part to an integer
    let fractional_part_as_int = fractional_value.parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Invalid fractional part"))?;

    // Overall result taking into account accuracy
    let factor = 10_i64.pow(precision);
    let mut result = integer_part * factor + fractional_part_as_int * if integer_part < 0 { -1 } else { 1 };

    if result > 0 && negative { 
        result = result * -1;
    }
    
    Ok(result)
}

pub fn stringed_float_to_i64_cached(s: &str, precision: u32, cache: &mut HashMap<String, i64>) -> anyhow::Result<i64> {
    if cache.contains_key(s) {
        Ok(*cache.get(s).unwrap())
    } else {
        let val = stringed_float_to_i64(s, precision)?;
        cache.insert(s.to_owned(), val);

        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use crate::biome::climate::decimal::stringed_float_to_i64;

    #[test]
    fn test_stringed_float_to_i32() {
        assert_eq!(0, stringed_float_to_i64("0.0", 1).unwrap());
        assert_eq!(0, stringed_float_to_i64("0", 1).unwrap());
        assert_eq!(50000000, stringed_float_to_i64("5", 7).unwrap());
        assert_eq!(7, stringed_float_to_i64("7.5", 0).unwrap());
        assert_eq!(-12, stringed_float_to_i64("-1.2", 1).unwrap());
        
        assert_eq!(-12000, stringed_float_to_i64("-1.2", 4).unwrap());
        assert_eq!(30500, stringed_float_to_i64("3.05", 4).unwrap());
        assert_eq!(7000, stringed_float_to_i64("0.7", 4).unwrap());
        assert_eq!(25354, stringed_float_to_i64("2.5354", 4).unwrap());
        assert_eq!(-4500, stringed_float_to_i64("-0.45", 4).unwrap());

        assert_eq!(0, stringed_float_to_i64("0.0001", 3).unwrap());
        assert_eq!(1, stringed_float_to_i64("0.0001", 4).unwrap());
        
        assert!(stringed_float_to_i64("a20.6", 4).is_err());
        assert!(stringed_float_to_i64("20.a6", 4).is_err());
        assert!(stringed_float_to_i64("20.6x", 4).is_err());
    }
}
