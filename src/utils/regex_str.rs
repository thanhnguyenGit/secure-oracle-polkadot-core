
static EXPORT_FUNC_RGX : &str =  r"export\s+function\s+(\w+)\s*\((.*?)\)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*\{";
static EXPORT_CLASS_RGX : &str =  r"export\s+class\s+(\w+)\s*\{";
static EXPORT_CONSTRUCTOR_RGX : &str = r"constructor\s*\((.*?)\)\s*\{";
static METHOD_RGX : &str =  r"(\w+)\s*\((.*?)\)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*\{";
static PARAMS_RGX : &str =  r"(public\s+)?(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)";
static VARS_RGX : &str = r"export\s+const\s+(\w+)\s*:\s*(\w+|\Array<\w+>|[\w<>]+)\s*=\s*[^;]+;";
static DOCS_RGX : &str = r"/\*\*\s*(.*?)\s*\*/";