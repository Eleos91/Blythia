var x: u64 = 10
def myfun() -> void:
  print_int(x)
  var x: u64 = 50
  print_int(x)

myfun()
x = 20
myfun()

def abc(x: u64,y: u64, z: u64) -> u64:
  print_int(x)
  print_int(y)
  print_int(z)
  x = 0
  y = 1
  z = 2
  print_int(x)
  print_int(y)
  print_int(z)
  return x + y + z

var ret: u64 = abc(10,20,30) + 1000
print_int(ret)
abc(100,200,300)
print_int(x)

def test(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64, h: u64, i: u64) -> void:
  print_int(a + b + c + d + e + f + g + h + i)

test(1,2,3,4,5,6,7,8,9)
