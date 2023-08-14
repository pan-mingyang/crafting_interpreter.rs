kws = '''
    Let, Func, Class,
    If, Else, While, For, In, Brake, Continue,
    And, Or, Not, Int, Str, Float, Bool,
    True, False, Nil,
    Import, Return, Kself,
    Print, 
'''

kws = map(lambda x: (x.strip().lower(), x.strip()), kws.split(','))

for kw, enum in kws:
    
    if kw == 'kself':
        kw = 'self'
    
    print(f'"{kw}" => Token::Keyword(Keyword::{enum}),')



