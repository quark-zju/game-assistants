// hand written multiple line grep '<level ... /level>'

#include <cstdio>
#include <string>
#include <cstring>

// 0F 00 00 00 NAME 00 76 ('v') 4B ('k') 00 00 "<levels>"
// 0E 00 00 00 NAME 00 00 77 4B 00 00 "<levels>"
// 0F 00 00 00 NAME 00 6B 06 00 00 "<level version="


using namespace std;

char ch;
string buf;
int state = 0;
char approx_name[64] = {0}; // approx level name

bool isname(char x) {
  return isprint(x) && (isalnum(x) || strchr("-_", x));
}

string parse_approx_name() {
  int i = sizeof(approx_name) - 9;
  while (i > 0 && isname(approx_name[i])) i--;

  string result;
  while (++i && i < (int)sizeof(approx_name)) {
    char ch = approx_name[i];
    if (!isname(ch)) break;
    result += ch;
  }
  return result;

}

int main(int argc, char const *argv[]) {
  // not started: 0
  // <: 1
  // l: 2
  // e: 3
  // v: 4
  // e: 5
  // l: 6
  //  : 7
  // v: 8
  // e: 9
  // r: 10
  // ...: 1024
  // /: 1025
  // l: 1026
  // e: 1027
  // v: 1028
  // e: 1029
  // l: 1030
  // >: 1031
  while (scanf("%c", &ch) == 1) {
    switch (ch) {
      case '<':
        if (state == 0) state++;
        else state = state & 1024;
        break;
      case 'l':
        if (state == 1 || state == 5 || state == 1025 || state == 1029) state++;
        else state = state & 1024;
        break;
      case 'e':
        if (state == 2 || state == 4 || state == 8 || state == 1026 || state == 1028) state++;
        else state = state & 1024;
        break;
      case 'v':
        if (state == 3 || state == 7 || state == 1027) state++;
        else state = state & 1024;
        break;
      case '/':
        if (state == 1024) state++;
        else state = state & 1024;
        break;
      case 'r':
        if (state == 9) {
          buf = "<!-- " + parse_approx_name() + " -->\n" "<level ve";
          state = 1024;
        } else state = state & 1024;
        break;
      case ' ':
        if (state == 6) state++;
        else state = state & 1024;
        break;
      case '>':
        if (state == 1030) {
          state = 0;
          buf += ch;
          printf("%s\n", buf.c_str());
        } else state = state & 1024;
        break;
      default:
        state &= 1024;
    }
    if (state == 0) {
      memmove(approx_name, approx_name + 1, sizeof(approx_name) - 1);
      approx_name[sizeof(approx_name) - 1] = ch;
    }
    if (state & 1024) buf += ch;
    // printf("%c %d\n", ch, state);
  }
  return 0;
}
