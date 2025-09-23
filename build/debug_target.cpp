#include <iostream>
#include <string>

// Minimal test without LLVM includes to isolate the issue
int main() {
  std::cout << "Testing basic functionality...\n";
  
  // Test basic string operations
  std::string test = "NeoVM Target Test";
  std::cout << "String test: " << test << "\n";
  
  // Test basic arithmetic
  int a = 2, b = 3;
  int sum = a + b;
  std::cout << "Arithmetic test: " << a << " + " << b << " = " << sum << "\n";
  
  std::cout << "Basic functionality works\n";
  return 0;
}
