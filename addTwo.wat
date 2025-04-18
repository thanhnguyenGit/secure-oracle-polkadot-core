(module
 (type $0 (func (param i32 i32) (result i32)))
 (type $1 (func (result i32)))
 (global $addTwo/url i32 (i32.const 1056))
 (memory $0 1)
 (data $0 (i32.const 1036) "\1c")
 (data $0.1 (i32.const 1048) "\02\00\00\00\0c\00\00\00G\00A\00Y\00V\00K\00L")
 (export "url" (global $addTwo/url))
 (export "add" (func $addTwo/add))
 (export "shout" (func $addTwo/shout))
 (export "memory" (memory $0))
 (func $addTwo/add (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  local.get $1
  i32.add
 )
 (func $addTwo/shout (result i32)
  i32.const 1056
 )
)
