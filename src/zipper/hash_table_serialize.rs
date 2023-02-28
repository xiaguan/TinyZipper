// struct TableElement {
//     key : u8,
//     // value is an array of two i8
//     value : [u8; 2],
// }

struct SerializeTable {
    table : Vec<TableElement>,
}

// impl the serialize trait and the deserialize func
impl SerializeTable {

    // the value of the map is a string that only contains '1' and '0'
    // we need to convert it to an array of u8
    fn New(map : HashMap<u8,String>) -> SerializeTable {
        let mut table = Vec::new();
        for (key,value) in map {
            let mut value_array = [0u8;2];
            for i in 0..value.len() {
                value_array[i] = value.as_bytes()[i];
            }
            table.push(TableElement{key,value:value_array});
        }
        SerializeTable{table}
    }

    fn serialize(&self,output_buf : &mut Vec<u8> )  {
        for i in 0..self.table.len() {
            output_buf.push(self.table[i].key);
            output_buf.push(self.table[i].value[0]);
            output_buf.push(self.table[i].value[1]);
        }
    }

    fn deserialize(input_buf : &Vec<u8>) -> HashMap<u8,String> {
        let mut map = HashMap::new();
        let mut i = 0;
        while i < input_buf.len() {
            let key = input_buf[i];
            let value = String::from_utf8(vec![input_buf[i+1],input_buf[i+2]]).unwrap();
            map.insert(key,value);
            i += 3;
        }
        map
    }

}

#[cfg(test)]
mod tests{
    use crate::zipper::zip_file_header;
    use std::collections::HashMap;

    #[test]
    fn check_serialize_then_deserialize(){
        // construct a test HaspMap<u8,String>
        let mut testMap = HashMap::new();
        
        let first_key : u8 = 56;
        let first_value : String = String::from("011101");
        
        let second_key : u8  = 26;
        let second_value : String = String::from("1101");

        // insert two k,v pairs 
        

        let test_serialize_table = zip_file_header::SerializeTable::New(testMap); 
        
    }
}