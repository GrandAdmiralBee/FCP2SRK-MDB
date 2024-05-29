#include <iostream>

int main() {
  std::string name1 = "Name1";
  std::string name2 = "Name2";
  std::string name3 = "Name3";

  if (1)
    qCritical() << "Err1" << name1 << name2;
  qInfo() << "Err2" << name3;
}
