var x = 10
def myfun():
  print_int(x)
  var x = 50
  print_int(x)

myfun()
x = 20
myfun()

def abc(x,y,z):
  print_int(x)
  print_int(y)
  print_int(z)
  x = 0
  y = 1
  z = 2
  print_int(x)
  print_int(y)
  print_int(z)

abc(10,20,30)

print_int(x)