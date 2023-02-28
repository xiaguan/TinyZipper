use std::{collections::HashMap};

pub const HEADER_SIZE : usize = 2;
pub const ELEMENT_SIZE : usize = 4;

struct TableElement {
    key : u8,
    value_size : u8,
    value : [u8; 2],
}

pub struct SerializeTable {
    table : Vec<TableElement>,
}

// impl the serialize trait and the deserialize func
impl SerializeTable {

    
    // The value of the map is a string that only contains '1' and '0'.
    // We need to convert it to an array of u8.
    pub fn new(map : &HashMap<u8,String>) -> SerializeTable {
        let mut table = Vec::new();
        for (key,value) in map {
            let mut value_array = [0u8;2];
            // Format the value into binary form into the value_array.
            for i in 0..value.len() {
                value_array[i/8] |= (value.as_bytes()[i] - 48) << (7 - i % 8);
            }
            table.push(TableElement{key : *key,value_size : value.len() as u8,value:value_array});
        }
        SerializeTable{table}
    }

    pub fn serialize(&self,output_buf : &mut Vec<u8> ,start_index : &mut usize){
        let size = self.table.len() as u16;
        output_buf[*start_index] = (size >> 8) as u8;
        output_buf[*start_index+1] = (size & 0xff) as u8;
        let mut i = *start_index+HEADER_SIZE;
        for element in &self.table {
            output_buf[i] = element.key;
            output_buf[i+1] = element.value_size;
            output_buf[i+2] = element.value[0];
            output_buf[i+3] = element.value[1];
            i += ELEMENT_SIZE;
        }
        *start_index = i;
    }

    pub fn deserialize(input_buf : &Vec<u8>) -> Result<HashMap<u8,String>,String> {
        let mut map = HashMap::new();
        if input_buf.len() < 4 {
            return Err("Input buffer is too short".to_string());
        }
        let table_size = (((input_buf[0] as u16) << 8) | (input_buf[1] as u16)) as usize;
        let mut i = HEADER_SIZE as usize;
        
         while i < table_size * ELEMENT_SIZE + HEADER_SIZE  {
            if i + 2 >= input_buf.len(){
                return Err("Input buffer is too short".to_string());
            }
            let key = input_buf[i];
            let value_size = input_buf[i+1] as usize;
            let mut value = String::new();
            for j in 0..value_size {
                value.push_str(if input_buf[i+2+j/8] & (1 << (7 - j % 8)) != 0 {"1"} else {"0"});
            }
            map.insert(key,value);
            i += ELEMENT_SIZE;
        }
        Ok(map)
    }

}

#[cfg(test)]
mod tests{
    use crate::zipper::hash_table_serialize;
    use std::collections::HashMap;

    #[test]
    fn check_serialize_then_deserialize(){
        // construct a test HaspMap<u8,String>
        let mut test_map = HashMap::new();
        
        let first_key : u8 = 56;
        let first_value : String = String::from("011101");
        
        let second_key : u8  = 26;
        let second_value : String = String::from("1101");

        let thrid_key : u8 = 119;
        let thrid_value : String = String::from("111");

        // insert two k,v pairs 
        test_map.insert(first_key,first_value);
        test_map.insert(second_key,second_value);
        test_map.insert(thrid_key,thrid_value);

        let mut output_buf:Vec<u8> = Vec::new();

        output_buf.resize(10000,0);


        let test_serialize_table = hash_table_serialize::SerializeTable::new(&test_map);
        
        // serialize the table
        let mut start_index = 0;
        let serialized_table = test_serialize_table.serialize(&mut output_buf,&mut start_index);
        // check the size 
        assert_eq!(start_index,hash_table_serialize::HEADER_SIZE + test_map.len() * hash_table_serialize::ELEMENT_SIZE);
        // debug the output buf 
        for i in 0..start_index {
            print!("{:02x} ",output_buf[i]);
        }
        println!();
        
        println!("serialized table {:?}",serialized_table);

        // deserialize the table
        let deserialized_table = hash_table_serialize::SerializeTable::deserialize(&output_buf).unwrap();
        println!("deserialized table {:?}",deserialized_table);
        // check every k,v pair in deserialized table is the same as the original one
        for (key,value) in deserialized_table {
            assert_eq!(test_map.get(&key).unwrap(),&value);
        }
    }
}