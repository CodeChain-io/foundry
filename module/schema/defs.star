firstWord = "[A-Za-z][a-z0-9]*|[A-Z][A-Z0-9]*"
word = "[a-z0-9]+|[A-Z0-9]+"
ident = "{}(-{})*".format(firstWord, word)
name = '^{}$'.format(ident)
qname = '^{id}(.{id})+$'.format(id=ident)
hash = '[a-f0-9]{256}'