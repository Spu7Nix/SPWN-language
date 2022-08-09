// macro_rules! ops {
//     ($($name:ident)*) => {
//         mod Operators {
//             // .................name
//             pub struct Operator(pub String);

//             $(
//                 const $name: Operator = Operator(stringify!($name));
//             )*
//         }
//     };
// }

// ops! {
//     Add
// }
