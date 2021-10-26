#[macro_export] 
macro_rules! set_traits {
    {
        trait $tname:ident { $($tbody:tt)* }
        $(
            [$sname:tt]
            $sfunc:item
        )*
    } => {
        trait $tname {
            $($tbody)*
        }
        
        $(
            impl $tname for $sname {
                $sfunc
            }
        )*
    }
}

pub struct MinifyOptions {
    pub keep: Vec<String>,
    pub clear_types: bool,
}