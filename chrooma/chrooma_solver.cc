#include <map>
#include <set>
#include <vector>
#include <cstdio>
#include <cassert>
#include <cstring>
#include <deque>

using namespace std;

const int BALL_0 = '0';
const int BALL_1 = '1';
const int BALL_2 = '2';
const int BALL_3 = '3';
const int BALL_4 = '4';
const int WALL = ' ';
const int BLANK = '.';
const int RIGHT = '>';

const int DIRECTION_DY[] = {0, 1, 0, -1};
const int DIRECTION_DX[] = {1, 0, -1, 0};
const char DIRECTION_CHAR[] = ">v<^ ";

struct BaseBoard {
  int height, width;
  int* cells;

  void init(int height, int width) {
    this->height = height;
    this->width = width;
    cells = new int[height * width];
  }

  ~BaseBoard() {
    if (cells) {
      delete[] cells;
      cells = nullptr;
    }
  }

  int at(int y, int x) const {
    return cells[y * width + x];
  }

  void set(int y, int x, int v) {
    cells[y * width + x] = v;
  }

  size_t cells_size() const {
    return sizeof(int) * width * height;
  }

  bool operator < (const BaseBoard& rhs) const {
    assert(height == rhs.height && width == rhs.width);
    int ret = memcmp(cells, rhs.cells, cells_size());
    return (ret < 0);
  }

  BaseBoard() { }

  private:

  BaseBoard(const BaseBoard& rhs) { }
  BaseBoard(const BaseBoard&& rhs) { }
  BaseBoard& operator=(const BaseBoard& rhs) { return *this; }
  BaseBoard& operator=(const BaseBoard&& rhs) { return *this; }

};

struct Board : public BaseBoard {
  map<int, int> goto_table;
  vector<int> unlock_positions;
};

Board* board;

struct State : public BaseBoard {
  Board* board;
  State* parent;
  int step;
  int last_direction;
  bool fail;

  State *move_state(int direction) {
    State * next = new State();
    next->copy_from(*this);
    next->last_direction = direction;
    next->move_now(direction);
    return next;
  }

  void copy_from(State& rhs) {
    init(rhs.height, rhs.width);
    step = rhs.step + 1;
    parent = &rhs;
    fail = rhs.fail;
    board = rhs.board;
    memcpy(cells, rhs.cells, rhs.cells_size());
  }

  bool get_board_unlocked() {
    for (auto& p : board->unlock_positions) {
      if (cells[p]) return true;
    }
    return false;
  }

  void move_now(int direction) {
    int dy = DIRECTION_DY[direction];
    int dx = DIRECTION_DX[direction];
    int pending_direction = -1;

    int y_begin = dy >= 0 ? 0 : height - 1;
    int y_step = dy >= 0 ? 1 : -1;
    int y_end = height - 1 - y_begin + y_step;
    int x_begin = dx >= 0 ? 0 : width - 1;
    int x_step = dx >= 0 ? 1 : -1;
    int x_end = width - 1 - x_begin + x_step;

    bool moving = false;
    bool first = true;
    bool board_unlocked = get_board_unlocked();
    killed_positions.clear();
    do {
      moving = false;
      for (int y = y_begin; y != y_end; y += y_step) {
        for (int x = x_begin; x != x_end; x += x_step) {
          int v = at(y, x);
          // is it a ball?
          if (v < '0' || v > '9') continue;
          int y_dest = y + dy, x_dest = x + dx;
          // dest avail?
          if (y_dest >= height || x_dest >= width || x_dest < 0 || y_dest < 0) continue;
          // dest is space?
          int board_dest = board->at(y_dest, x_dest);
          if (board_dest == 'x') {
            // resolve unblock
            // keep unlock if it is killed position
            // see hard-20
            bool local_unlocked = board_unlocked;
            if (!local_unlocked && killed_positions.count(y_dest * width + x_dest)) {
              local_unlocked = true;
            }
            board_dest = local_unlocked ? '.' : ' ';
          }
          if (board_dest == ' ' || at(y_dest, x_dest)) continue;
          // trapped in direction sign?
          const char * board_direction = strchr(DIRECTION_CHAR, board->at(y, x));
          if (board_direction && (board_direction - DIRECTION_CHAR) != direction) {
            // trapped (see medium-23)
            continue;
          }
          // move!
          set(y_dest, x_dest, v);
          set(y, x, 0);
          moving = true;
          // touch something?
          switch (board->at(y_dest, x_dest)) {
            case '*':
              // kill instantly
              set(y_dest, x_dest, 0);
              break;
            case '@':
              {
                // wrap, goto another '@'
                int pos = board->goto_table[y_dest * width + x_dest];
                set(y_dest, x_dest, 0);
                cells[pos] = v;
              }
              break;
            case '>':
              // next move: '>'
              pending_direction = 0;
              break;
            case 'v':
              pending_direction = 1;
              break;
            case '<':
              pending_direction = 2;
              break;
            case '^':
              pending_direction = 3;
              break;
          }
        }
      }
      if (!moving && first) {
        // nothing moved
        fail = true;
        return;
      }
      if (!moving && pending_direction < 0) {
        moving = check_adjacent();
        if (moving) {
          // if some dots are killed, recalculate board_unlocked
          // see hard-13
          bool new_board_unlocked = get_board_unlocked();
          board_unlocked = get_board_unlocked();
        }
      }
      first = false;
    } while (moving);

    if (pending_direction >= 0) {
      move_now(pending_direction);
    }
  }

  std::set<int> killed_positions;

  bool check_adjacent() {
    std::set<int> killed;
    std::map<int, int> left;
    for (int y = 0; y < height; ++y) {
      for (int x = 0; x < width; ++x) {
        int v = abs(at(y, x));
        if (v == 0) continue;
        if (y < height - 1 && abs(at(y, x)) == abs(at(y + 1, x))) {
          killed.insert(v);
          set(y, x, -v);
          set(y + 1, x, -v);
        }
        if (x < width - 1 && abs(at(y, x)) == abs(at(y, x + 1))) {
          killed.insert(v);
          set(y, x, -v);
          set(y, x + 1, -v);
        }
      }
    }

    // remove all, check fail
    for (int y = 0; y < height; ++y) {
      for (int x = 0; x < width; ++x) {
        int v = at(y, x);
        if (v > 0) {
          left[v]++;
        } else if (v < 0) {
          killed_positions.insert(y * width + x);
          set(y, x, 0);
        }
      }
    }
    // enter "fail" state
    if (left.empty()) {
      // success
      printf("SUCCESS !\n");
      print_recursively();
      exit(0);
    }
    for (auto &p : left) {
      if (p.second <= 1) {
        // left a single dot for color p.first, fail immediately
        fail = true;
        return false;
      }
    }
    return !killed.empty();
  }

  void print() {
    printf("State step: %d %s\n", step, fail ? "(failed)" : "");
    for (int y = 0; y < height; ++y) {
      for (int x = 0; x < width; ++x) {
        int v = at(y, x);
        if (v > 0) putchar(v);
        else putchar(board->at(y, x));
      }
      putchar('\n');
    }
  }

  void print_recursively() {
    State * pstate = this;
    string steps = "";
    while (pstate) {
      char direction = DIRECTION_CHAR[pstate->last_direction];
      pstate->print();
      printf("------- %c -------\n", direction);
      pstate = pstate->parent;
      steps = string(" ") + direction + steps;
    }
    printf("Steps: %s\n", steps.c_str());
  }

  State() {
    last_direction = 4;
    fail = false;
  }
};

const int STEP_LIMIT = 10;

void search(State* initial_state) {
  deque<State*> queue;
  set<State*> seen;

  seen.insert(initial_state);
  queue.push_back(initial_state);

  for (;;) {
    if (queue.empty()) {
      printf("NO SOLUTION\n");
      exit(2);
    }
    State *pstate = *queue.begin();
    if (pstate->step > STEP_LIMIT) {
      printf("STEP LIMIT EXCEEDED\n");
      exit(1);
    }
    queue.pop_front();

    for (int d = 0; d < 4; ++d) {
      State * next_state = pstate->move_state(d);

      if (seen.count(next_state) || next_state->fail) {
        delete next_state;
      } else {
        seen.insert(next_state);
        queue.push_back(next_state);
      }
    }
    
  }
}

int main(int argc, char const *argv[]) {
  char ch;
  int width = -1, height = -1;
  string buf = "";
  while (scanf("%c", &ch) == 1) {
    switch (ch) {
      case '\n': case '\r':
        if (width < 0) width = buf.length();
        break;
      default:
        buf += ch;
    }
  }

  height = (buf.length() / width);
  if (height * width != (int)buf.length()) {
    printf("Incorrect board size\n");
    exit(1);
  }

  board = new Board();
  board->init(height, width);

  State * state = new State();
  state->parent = nullptr;
  state->init(height, width);
  state->board = board;
  state->step = 0;

  map<char, vector<int> > special_positions;
  for (int y = 0; y < height; ++y) {
    for (int x = 0; x < width; ++x) {
      int pos = y * width + x;
      int v = buf[pos];
      if (v >= '0' && v <= '9') {
        state->set(y, x, v);
        board->set(y, x, '.');
      } else {
        board->set(y, x, v);
      }
      if (strchr("@ox", v)) {
        special_positions[v].push_back(pos);
      }
    }
  }
  if (special_positions['@'].size() == 2) {
    auto& goto_pos = special_positions['@'];
    board->goto_table[goto_pos[0]] = goto_pos[1];
    board->goto_table[goto_pos[1]] = goto_pos[0];
  }
  if (special_positions['o'].size() > 0) {
    board->unlock_positions = special_positions['o'];
  }
  
  search(state);

  // string steps = "v";
  // state->print();
  // for (char c : steps) {
  //   int direction = strchr(DIRECTION_CHAR, c) - DIRECTION_CHAR;
  //   state->move_now(direction);
  //   printf("---- %c ----\n", c);
  //   state->print();
  // }

  return 0;
}
