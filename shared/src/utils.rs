pub fn get_response_json(code:i32,msg:String,data:String)->String{
    return format!(
r#"{{
"code":{},
"message":"{}",
"data":{}
}}"#,code,msg,data)
}

pub fn normalize_vector<T>(page_no:i32,page_size:i32,from:&Vec<T>)->Vec<T>
where 
T:Clone
{    
    //normalize
    let first = page_size*(page_no-1);
    let mut last = page_no*page_size-1;
    let len = from.len() as i32;
    if last > len  {
        last = len;
    }
    println!("normalize_vector count {},first {},last {}", len,first,last);
    let ret:Vec<T>;
    if first < len && last > first {
        ret = (&from[(first as usize)..(last as usize)]).to_vec();        
    }
    else {       
        ret = Vec::new();
    }
    ret
}