use std::collections::BTreeMap;
use mysql;
use once_cell::sync::Lazy;
use crate::config;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, App, HttpResponse, HttpServer, Responder,HttpRequest};
#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub productId: u64,
    pub productName: String,
    pub detailDesc: String,
    pub mainUrl: String,
    pub verifiedStatus: i32,
    pub saleStatus: i32,
    pub level_id:i32,
    pub prdt_type_id:i32,
    pub brand_id:i32,
}
impl PartialEq for ProductInfo {
    fn eq(&self, other: &Self) -> bool {
        self.productId == other.productId
    }
}
impl ProductInfo{
    pub fn new(id:u64,name:String,desc:String,url:String,verify:i32,lid:i32,prdt:i32,b_id:i32)->ProductInfo{
        ProductInfo{
            productId:id,
            productName:name,
            detailDesc:desc,
            mainUrl:url,
            verifiedStatus:verify,
            saleStatus:0,
            level_id:lid,
            prdt_type_id:prdt,
            brand_id:b_id,
        }
    }
}
#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct Sku {
    pub id: u64,
    pub product_id:u64,
    pub sku: String,
    pub detail: String,
    pub custom_price:f32,
}
impl PartialEq for Sku {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Sku{
    fn update(&mut self,other:&Self){
        self.detail = other.detail.clone();
        self.custom_price = other.custom_price;
    }
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct Product {
    info:ProductInfo,
    skus:BTreeMap<u64,Sku>,
}
impl Product{
    pub fn new_less(i:ProductInfo)->Product{
        Product{
            info:i,
            skus:BTreeMap::new(),
        }
    }
    pub fn new(i:ProductInfo,s:BTreeMap<u64,Sku>)->Product{
        Product{
            info:i,
            skus:s,
        }
    }
    pub fn get_id(&self)->u64{
        self.info.productId
    }
    pub fn get_info(&self)->ProductInfo{
        self.info.clone()
    }
    pub fn get_ref_info(&self)->Option<&ProductInfo>{
        Some(&self.info)
    }
    pub fn get_skus(&self)->Vec<Sku>{
        let ret:Vec<Sku> = self.skus.values().cloned().collect();
        ret
    }
    ///更新sku，如果没有就添加
    pub fn update_sku(&mut self,_sku:Sku){
        if self.skus.contains_key(&_sku.id){
            self.skus.get_mut(&_sku.id).unwrap().update(&_sku);
        }
        else {
            self.skus.insert(_sku.id, _sku);
        }
    }
    pub fn set_skus(&mut self,_skus:Vec<Sku>)
    {
        self.skus.clear();
        for sk in _skus {
            self.skus.insert(sk.id, sk);
        }
    }
}
impl PartialEq for Product {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}
#[derive(Debug,Clone, Serialize, Deserialize)]
///上架产品
pub struct SaleProductInfo {
    pub productId: u64,
    monthBeforeIncome: i32,
    monthBeforeCharge: i32,
    saleCount: i32,
    selectionType: i32,
    stockTotal: i32,
}

impl SaleProductInfo {
    pub fn new(id:u64,mbi:i32,mbc:i32,sc:i32,_type:i32,st:i32)->SaleProductInfo{
        SaleProductInfo{
            productId: id,
            monthBeforeIncome: mbi,
            monthBeforeCharge: mbc,
            saleCount: sc,
            selectionType: _type,
            stockTotal: st,
        }
    }
    pub fn update(&mut self, other: &Self) {
        assert_eq!(self.productId, other.productId);
        self.monthBeforeCharge = other.monthBeforeCharge;
        self.monthBeforeIncome = other.monthBeforeIncome;
        self.saleCount = other.saleCount;
        self.selectionType = other.selectionType;
        self.stockTotal = other.stockTotal;
    }
}
impl PartialEq for SaleProductInfo {
    fn eq(&self, other: &Self) -> bool {
        self.productId == other.productId
            && self.monthBeforeCharge == other.monthBeforeCharge
            && self.monthBeforeIncome == other.monthBeforeIncome
            && self.saleCount == other.saleCount
            && self.selectionType == other.selectionType
            && self.stockTotal == other.stockTotal
    }
}
#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct SaleProductData {
    product:ProductInfo,
    monthBeforeIncome: i32,
    monthBeforeCharge: i32,
    saleCount: i32,
    selectionType: i32,
    stockTotal: i32,
}
impl SaleProductData {
    pub fn new(prod:ProductInfo,sale_info:SaleProductInfo)->SaleProductData{
        SaleProductData{
            product:prod,
            monthBeforeIncome:sale_info.monthBeforeIncome,
            monthBeforeCharge:sale_info.monthBeforeCharge,
            saleCount:sale_info.saleCount,
            selectionType:sale_info.selectionType,
            stockTotal:sale_info.stockTotal,
        }
    }
}
static PRODUCT_POOL: Lazy<Mutex<mysql::Pool>> = Lazy::new(|| {
    Mutex::new(mysql::Pool::new(
        config::get("db_product").unwrap(),
    )
    .unwrap(),)
});

static S_PRODUCT_INFO: Lazy<Mutex<BTreeMap<u64, Product>>> =
Lazy::new(|| Mutex::new(BTreeMap::new()));


static S_SALE_PRODUCT: Lazy<Mutex<BTreeMap<u64, SaleProductInfo>>> =
Lazy::new(|| Mutex::new(BTreeMap::new()));



fn load_skus(pool:&mysql::Pool,product_id:&u64)->BTreeMap<u64,Sku>{
    let mut skus:BTreeMap<u64,Sku> = BTreeMap::new();
    println!("load sku for product {}", product_id);
    pool.prep_exec(
        "SELECT id,sku_id,custom_price,detail FROM product_sku WHERE product_id = ?",
            (product_id,),
        )
        .map(|result| {
            let raw_skus = result.map(|x| x.unwrap()).fold(Vec::new(), |mut v, row| {
                v.push(mysql::from_row::<(u64,Option<String>,f32,Option<String>)>(row));
                v
            });
            println!("product {} sku count {}",product_id,raw_skus.len());
            for rs in &raw_skus {
                let _sku = Sku{
                    id:rs.0,
                    product_id:*product_id,
                    sku:rs.1.as_ref().unwrap_or(&String::default()).to_string(),
                    detail:rs.3.as_ref().unwrap_or(&String::default()).to_string(),
                    custom_price:rs.2,
                };
                skus.insert(rs.0,_sku);
            }
        }).ok().or_else(||{println!("fail to load skus for product {} from product_sku",product_id);None});
    skus
}
///厂商产品是不会缓存到产品缓存的，需要厂商自己缓存
pub fn get_producer_products(producer_id:&u64)->Vec<Product>{
    let mut products:Vec<Product> = Vec::new();
    let pool = PRODUCT_POOL.lock().unwrap();
    pool.prep_exec(
        "SELECT id,product_name,detail_title, main_url,verified_status,level_id,prdt_type_id,brand_id FROM product_info_old WHERE sources_id = ?",
            (producer_id,),
        )
        .map(|result| {
            let raw_products = result.map(|x| x.unwrap()).fold(Vec::new(), |mut v, row| {
                v.push(mysql::from_row::<(u64,Option<String>,Option<String>,Option<String>,i32,i32,i32,i32)>(row));
                v
            });
            println!("producer {} products count {}",producer_id,raw_products.len());
            for rp in &raw_products {
                println!("product {} name {}",rp.0,rp.1.as_ref().unwrap());
                //load skus
                products.push(Product::new(ProductInfo::new(
                    rp.0,
                    rp.1.as_ref().unwrap_or(&String::from("")).clone(),
                    rp.2.as_ref().unwrap_or(&String::from("")).clone(),
                    rp.3.as_ref().unwrap_or(&String::from("")).clone(),
                    rp.4,
                    rp.5,
                    rp.6,
                    rp.7
                ),load_skus(&pool, &rp.0)));
            }
        }).ok().or_else(||{println!("fail to load self products for producer {} from product_info_old",producer_id);None});
    products
}
///自有产品是不会缓存到产品缓存的，需要租户自己缓存
pub fn get_tenant_products(tent_id:&u64)->Vec<Product>{
    let mut products:Vec<Product> = Vec::new();
    let pool = PRODUCT_POOL.lock().unwrap();
    pool.prep_exec(
        "SELECT id,product_name,detail_title, main_url,verified_status,level_id,prdt_type_id,brand_id FROM product_info_old WHERE tent_id = ?",
            (tent_id,),
        )
        .map(|result| {
            let raw_products = result.map(|x| x.unwrap()).fold(Vec::new(), |mut v, row| {
                v.push(mysql::from_row::<(u64,Option<String>,Option<String>,Option<String>,i32,i32,i32,i32)>(row));
                v
            });
            println!("tenant {} products count {}",tent_id,raw_products.len());
            for rp in &raw_products {
                println!("product {} name {}",rp.0,rp.1.as_ref().unwrap());
                //load skus
                
                products.push(Product::new(ProductInfo::new(
                    rp.0,
                    rp.1.as_ref().unwrap_or(&String::from("")).clone(),
                    rp.2.as_ref().unwrap_or(&String::from("")).clone(),
                    rp.3.as_ref().unwrap_or(&String::from("")).clone(),
                    rp.4,
                    rp.5,
                    rp.6,
                    rp.7
                ),load_skus(&pool, &rp.0)));
            }
        }).ok().or_else(||{println!("fail to load self products for tenant {} from product_info_old",tent_id);None});
    products
}
pub fn get_product(product_id:&u64)->Option<Product>
{
    let mut prod:Option<Product> = None;
    {
        prod = S_PRODUCT_INFO.lock().unwrap().get(product_id).and_then(|p|Some(p.clone())).or_else(||None);
    }
    if prod == None{
        let pool = PRODUCT_POOL.lock().unwrap();
        if let Some(row) = pool
            .first_exec(
                "SELECT product_name,detail_title, main_url,verified_status,level_id,prdt_type_id,brand_id FROM product_info_old WHERE product_id = ?",
                (product_id,),
            )
            .unwrap_or(None)
        {
            let (product_name,desc, main_url,verify,lid,prdt,b_id) = mysql::from_row::<(Option<String>, Option<String>,Option<String>,i32,i32,i32,i32)>(row); 
            println!("load product sku for product {},{}", product_id,product_name.as_ref().unwrap_or(&String::default()));           
            let cache =Product::new(ProductInfo::new(
                *product_id,
                product_name.unwrap_or(String::default()),
                desc.unwrap_or(String::default()),
                main_url.unwrap_or(String::default()),
                verify,
                lid,
                prdt,
                b_id,
            ),load_skus(&pool, product_id));
            prod = Some(cache.clone());
            S_PRODUCT_INFO.lock().unwrap().insert(*product_id,cache);
        }
    }
    prod
}
pub fn get_product_skus(product_id:&u64)->Vec<Sku>{     
    if let Some(p) = get_product(product_id){
        return p.get_skus();
    }
    Vec::new()
}
///获取单个产品信息
pub fn get_product_info(id:&u64)->Option<ProductInfo>{
    if let Some(p) = get_product(id)
    {
        return Some(p.get_info());
    }
    None
}
///获取在售产品数据
pub fn get_sale_product(id:&u64)->Option<SaleProductInfo>{    
    let mut prod:Option<SaleProductInfo> = None;
    {
        prod = S_SALE_PRODUCT.lock().unwrap().get(&id).and_then(|p|Some(p.clone())).or_else(||None);
    }
    if prod == None{
        if let Some(_) = PRODUCT_POOL.lock().unwrap()
            .first_exec(
                "SELECT  FROM product_store_prod WHERE product_id = ?",
                (id,),
            )
            .unwrap_or(None)
        {
            //let store_id = mysql::from_row::<u64>(row);
            let cache = SaleProductInfo{
                productId:*id,
                monthBeforeIncome:0,
                monthBeforeCharge:0,
                saleCount:0,
                selectionType:0,
                stockTotal:0,
            };
            prod = Some(cache.clone());
            S_SALE_PRODUCT.lock().unwrap().insert(*id,cache);
        }
    }
    prod
}
///获取店铺上架产品列表
pub fn get_sale_products(shop_id:&u64)->Vec<SaleProductInfo>{
    let mut products:Vec<SaleProductInfo> = Vec::new();
    //load from db directly
    {
        PRODUCT_POOL.lock().unwrap().prep_exec(
            "SELECT product_id FROM product_store_prod WHERE store_id = ?",
                (shop_id,),
            )
            .map(|result| {
                let raw_products = result.map(|x| x.unwrap()).fold(Vec::new(), |mut v, row| {
                    v.push(mysql::from_row::<u64>(row));
                    v
                });
                println!("shop_id {} products count {}",shop_id,raw_products.len());
                for rp in &raw_products {
                    let product = SaleProductInfo{
                        productId:*rp,
                        monthBeforeIncome:0,
                        monthBeforeCharge:0,
                        saleCount:0,
                        selectionType:0,
                        stockTotal:0,
                    };
                    //缓存
                    {
                        let mut s_sale = S_SALE_PRODUCT.lock().unwrap();
                        if !s_sale.contains_key(rp){
                            s_sale.insert(*rp,product.clone());
                        }
                    }
                    products.push(product);
                }
            }).ok().or_else(||{println!("fail to load self products for tenant {} from product_info",shop_id);None});
    }
    products
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductImportSku{
    pub detail:String,
    pub sku:String,
    pub custom_price:i32,
}

pub fn import_self_product(tid:u64,product_id:u64,name:String,desc:String,url:String,sku:&Vec<ProductImportSku>)->bool{ 
    let mut ok = true;   
    PRODUCT_POOL.lock().unwrap().start_transaction(false, None, None)
    .and_then(|mut t| {        
        ok = t.prep_exec("INSERT INTO product_info_old (tent_id,sources_id,product_id,product_name,detail_title,main_url,created_time) VALUES (?,0,?,?,?,?,NOW())",
        (tid,product_id, name, desc, url,)).is_ok();   
        if ok {
            for _sku in sku {      
               ok = t.prep_exec("INSERT INTO product_sku (product_id,sku_id,detail,custom_price) VALUES(?,?,?,?)",(product_id,_sku.sku.clone(),_sku.detail.clone(),_sku.custom_price,)).is_ok();
               if !ok {
                   break;
               }
            }
        }
        if ok {
            let ret = t.commit();
            ok =   ret.is_ok();
            return ret;
        }
        else {
            ok = false;
            return t.rollback();
        }
    })
    .unwrap();
    ok
}

pub fn import_producer_product(pid:u64,product_id:u64,name:String,desc:String,url:String,sku:&Vec<ProductImportSku>)->bool{
    let mut ok = true;   
    PRODUCT_POOL.lock().unwrap().start_transaction(false, None, None)
    .and_then(|mut t| {        
        ok = t.prep_exec("INSERT INTO product_info_old (tent_id,sources_id,product_id,product_name,detail_title,main_url,created_time) VALUES (0,?,?,?,?,?,NOW())",
        (pid,product_id, name, desc, url,)).is_ok();   
        if ok {
            for _sku in sku {      
               ok = t.prep_exec("INSERT INTO product_sku (product_id,sku_id,detail,custom_price) VALUES(?,?,?,?)",(product_id,_sku.sku.clone(),_sku.detail.clone(),_sku.custom_price,)).is_ok();
               if !ok {
                   break;
               }
            }
        }
        if ok {
            let ret = t.commit();
            ok =   ret.is_ok();
            return ret;
        }
        else {
            ok = false;
            return t.rollback();
        }
    })
    .unwrap();
    ok
}