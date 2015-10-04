#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cassert>
#include <string>
#include <vector>
#include <list>
#include <deque>
#include <map>
#include <set>

void fatal(const char * msg) {
  puts(msg);
  exit(EXIT_FAILURE);
}

#define DEBUG_LEVEL 0
#define D(x) if (DEBUG_LEVEL >= x)

namespace geometry {
  typedef double real;

  enum Angle : int {
    ACUTE = -1,
    RIGHT = 0,
    OBTUSE = 1,
  };

  const real eps = 1e-6;

  inline bool is_zero(real x) { return x <= eps && x >= -eps; }

  template<typename T>
    T min(T a, T b) { return a < b ? a : b; }

  template<typename T>
    T max(T a, T b) { return a > b ? a : b; }

  struct Point {
    real x, y;
    Point(real x = 0, real y = 0) : x(x), y(y) {};

    real distance(Point p) {
      real dx = x - p.x;
      real dy = y - p.y;
      return sqrt(dx * dx + dy * dy);
    }

    real dot(Point p) {
      return x * p.x + y * p.y;
    }

    real cross(Point p) {
      return x * p.y - y * p.x;
    }

    Point minus(Point p) {
      return Point(x - p.x, y - p.y);
    }

    Point add(Point p) {
      return Point(x + p.x, y + p.y);
    }

    Point scale(real k) {
      return Point(x * k, y * k);
    }

    inline static Angle angle(Point& p1, Point& p2, Point& p3) {
      real v = (p3.x - p2.x) * (p2.x - p1.x) + (p3.y - p2.y) * (p2.y - p1.y);
      if (is_zero(v)) return Angle::RIGHT;
      else if (v < 0) return Angle::ACUTE;
      else return Angle::OBTUSE;
    }

    void print(FILE * fd = stdout) {
      fprintf(fd, "(%.4f, %.4f)", x, y);
    }
  };

  struct Line {
    real x1, y1, x2, y2;

    Line() {
      x1 = y1 = x2 = y2 = 0;
    }

    Line(Point p1, Point p2) {
      x1 = p1.x;
      y1 = p1.y;
      x2 = p2.x;
      y2 = p2.y;
    }

    Line(real x1, real y1, real x2, real y2) : x1(x1), y1(y1), x2(x2), y2(y2) {};

    inline Point *p1() {
      return reinterpret_cast<Point*>(&x1);
    }

    inline Point *p2() {
      return reinterpret_cast<Point*>(&x2);
    }

    void print(FILE * fd) {
      p1()->print(fd);
      fprintf(fd, "-");
      p2()->print(fd);
    }

    real distance(Point p) {
      real y2_y1 = y2 - y1;
      real x2_x1 = x2 - x1;
      real result = (y2_y1 * p.x - x2_x1 * p.y + x2 * y1 - y2 * x1)\
                    / sqrt(y2_y1 * y2_y1 + x2_x1 * x2_x1);
      return fabs(result);
    }

    real length() {
      real y2_y1 = y2 - y1;
      real x2_x1 = x2 - x1;
      return sqrt(y2_y1 * y2_y1 + x2_x1 * x2_x1);
    }
  };

  struct LineSegment: public Line {
    using Line::Line;

    real distance(Point p) {
      if (Point::angle(*p1(), *p2(), p) == Angle::OBTUSE) return p2()->distance(p);
      if (Point::angle(*p2(), *p1(), p) == Angle::OBTUSE) return p1()->distance(p);
      return static_cast<Line*>(this)->distance(p);
    }

    bool intersect(LineSegment& l, Point* out_p = nullptr) {
      // http://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect
      Point p = *p1();
      Point q = *(l.p1());
      Point r = Point(x2 - x1, y2 - y1);
      Point s = Point(l.x2 - l.x1, l.y2 - l.y1);
      if (is_zero(r.cross(s))) {
        if (is_zero(q.minus(p).cross(r))) { // collinear
          return max(x1, x2) >= min(l.x1, l.x2) && min(x1, x2) <= max(l.x1, l.x2) && max(y1, y2) >= min(l.y1, l.y2) && min(y1, y2) <= max(l.y1, l.y2);
        } else { // parallel & not-intersecting
          return false;
        }
        return is_zero(q.minus(p).cross(r)); // collinear or parallel & not-intersecting
      }
      real t = q.minus(p).cross(s) / (r.cross(s));
      real u = p.minus(q).cross(r) / (s.cross(r));
      if (out_p) {
        Point intersect_p = p.add(r.scale(t));
        out_p->x = intersect_p.x;
        out_p->y = intersect_p.y;
      }
      D(31) fprintf(stderr, "u = %g t = %g\n", u, t);
      return u >= 0 && u <= 1 && t >= 0 && t <= 1;
    }

    LineSegment& shorten(real shortenLen) {
      real l = length();
      real dx = (x2 - x1) / l * shortenLen;
      real dy = (y2 - y1) / l * shortenLen;
      x1 += dx;
      x2 -= dx;
      y1 += dy;
      y2 -= dy;
      return *this;
    }
  };

  struct Circle {
    real x, y, r;

    Circle(Point p, real r) : x(p.x), y(p.y), r(r) {};
    Circle(real x, real y, real r) : x(x), y(y), r(r) {};

    bool intersect(LineSegment& l) {
      return l.distance(Point(x, y)) <= r;
    }
  };
}

namespace xml { // naive xml processing. do not use it in production
  using geometry::real;

  std::string extractStr(const char * line, const char * field) {
    std::string needle = std::string(" ") + field + "=\"";
    const char * p = strstr(line, needle.c_str());
    if (!p) return "";

    p += strlen(field) + 3;
    const char * p2 = strchr(p, '"');
    if (!p2) return "";
    return std::string(p, p2 - p);
  }

  real extractReal(const char * line, const char * field) {
    return atof(extractStr(line, field).c_str());
  }

  int extractInt(const char * line, const char * field) {
    return atoi(extractStr(line, field).c_str());
  }

  int extractBool(const char * line, const char * field) {
    auto s = xml::extractStr(line, field);
    if (s.length() == 0) return 0;
    return s[0] == 't' || s[0] == 'T' || s[0] == '1' || s[0] == 'y' || s[0] == 'Y';
  }


  geometry::Point extractPoint(const char * line, const char * field) {
    auto s = extractStr(line, field).c_str();
    const char * p = strchr(s, ',');
    if (p) {
      return geometry::Point(atof(s), atof(p + 1));
    } else {
      return geometry::Point();
    }
  }
}

namespace transmission {
  using geometry::real;
  using geometry::Point;
  using xml::extractStr;
  using xml::extractInt;
  using xml::extractReal;
  using xml::extractPoint;
  using xml::extractBool;

  const int MAX_ELEMENTS = 21;

  int objSigCount = -1;
  int objTargetValue = -1;
  enum ObjectiveIndex { ObjAbsent = 0, ObjCrossWires = 1, ObjSigCount = 2, ObjTargetValue = 4 };
  int objSelected = ObjAbsent;

  enum ElementType: int {
    InvalidElement = -1,
    CellTransmitter = 0, // station ? yes
    ObjectiveCrossedWires,
    ObjectiveSignalCount,
    ObjectiveTargetValue,
    PlacedSignal, // ignore
    RadialTransmitter, // radius?
    Receiver,
    SignalBlock,
    SignalBlockCircle,
    SignalBlockHexagon,
    SignalBooster,  // Repeater ?
    SwapperTransmitter,
    Transceiver,
    Transmitter,
  };

  enum ElementGroup: int {
    InvalidColor = -1,
    Cable = 0,
    Exchange,
    Fibre,
    Wave,
  };

  ElementGroup extractElementGroup(const char * line, const char * field) {
    auto s = xml::extractStr(line, field);
    if (s == "Cable") return Cable;
    else if (s == "Exchange") return Exchange;
    else if (s == "Fibre") return Fibre;
    else if (s == "Wave") return Wave;
    return InvalidColor;
  }

  ElementType extractElementType(const char * line, const char * field) {
    auto s = xml::extractStr(line, field);
    if (s == "CellTransmitter") return CellTransmitter;
    else if (s == "ObjectiveCrossedWires") return ObjectiveCrossedWires;
    else if (s == "ObjectiveSignalCount") return ObjectiveSignalCount;
    else if (s == "ObjectiveTargetValue") return ObjectiveTargetValue;
    else if (s == "PlacedSignal") return PlacedSignal;
    else if (s == "RadialTransmitter") return RadialTransmitter;
    else if (s == "Receiver") return Receiver;
    else if (s == "SignalBlock") return SignalBlock;
    else if (s == "SignalBlockCircle") return SignalBlockCircle;
    else if (s == "SignalBlockHexagon") return SignalBlockHexagon;
    else if (s == "SignalBooster") return SignalBooster;
    else if (s == "SwapperTransmitter") return SwapperTransmitter;
    else if (s == "Transceiver") return Transceiver;
    else if (s == "Transmitter") return Transmitter;
    return InvalidElement;
  }

  struct State {
    char amounts[MAX_ELEMENTS];
    char left[MAX_ELEMENTS];
    char connected[MAX_ELEMENTS][MAX_ELEMENTS]; // connected[i][j]: connected from i to j, with color connected[i][j]
    char colorSwapped[MAX_ELEMENTS];

    bool operator < (const State& rhs) const {
      return memcmp(this, &rhs, sizeof(State)) < 0;
    }
  };
  State* currentState;
  struct Level;
  Level* currentLevel;

  char connectable[MAX_ELEMENTS][MAX_ELEMENTS];
  bool isWireBlockedByBlockersNow(int srcId, int dstId); // functions end with "Now" is dynamic, depends on current{Level,State}

  struct Element {
    ElementType type;
    ElementGroup color;
    int amount;
    int target;
    int id;
    Point pos;

    virtual void readXML(const char * p) = 0;

    // static properties
    virtual bool isSender() { return false; }
    virtual bool isReceiver() { return false; }
    virtual bool isColorFixed() { return true; }

    virtual bool canReceiveColor(ElementGroup color) { return this->color == color; }
    virtual real inBetweenRadius() { return 1; }
    virtual bool isInBetween(geometry::LineSegment& l) { return l.distance(pos) < inBetweenRadius(); }
    virtual bool isColorMatch(Element* dst) { return dst->canReceiveColor(color); }
    virtual bool isObjective() { return false; }
    virtual bool isBlock() { return false; }
    virtual bool isWireless() { return false; } // wireless: ignore when considering cross objectives
    virtual void init(std::vector<Element*>& elements) { }

    // currentState Involved
    virtual ElementGroup colorNow() { return this->color; }
    virtual bool canReceiveColorNow(ElementGroup color) { return this->color == color; }
    virtual bool canReceivePacketNow() { return canReceivePacketNumberNow() > 0; }
    virtual int canReceivePacketNumberNow() { return target - currentState->amounts[id]; }
    virtual bool hasExtraPacketNow() { return currentState->left[id] > 0; }
    virtual bool isFulfilled() { return currentState->amounts[id] == target; }
    virtual bool canConnectToNow(Element* dst) {
      if (!connectable[id][dst->id]) return false;
      if (currentState->connected[id][dst->id] || currentState->connected[dst->id][id]) return false;
      if (!hasExtraPacketNow() || !dst->canReceivePacketNow()) return false;
      if (!dst->canReceiveColorNow(colorNow())) return false;
      if (isWireBlockedByBlockersNow(id, dst->id)) return false;
      return true;
    }
    virtual bool willCrossIfConnectNow(Element* dst, std::vector<Element*>& elements) {
      // check cross wires
      geometry::LineSegment line(pos, dst->pos);
      line.shorten(1);  // 1 = element radius
      int n = elements.size();
      for (int i = 0; i < n; ++i) {
        if (elements[i]->isWireless()) continue;
        for (int j = i + 1; j < n; ++j) {
          if (elements[j]->isWireless()) continue;
          if (currentState->connected[i][j] + currentState->connected[j][i] == 0) continue;
          geometry::LineSegment lineTest(elements[i]->pos, elements[j]->pos);
          lineTest.shorten(1);
          if (lineTest.intersect(line)) {
            D(4) {
              fprintf(stderr, "      %d - %d will cross %d - %d", id, dst->id, i, j);
              D(5) {
                fprintf(stderr, ": ");
                line.print(stderr);
                fprintf(stderr, ", ");
                lineTest.print(stderr);
              }
              fprintf(stderr, "\n");
            }
            return true;
          }
        }
      }
      return false;
    }
    virtual int onConnectedNow(Element* src, std::vector<Element*>& elements) { return 0; }
    virtual int connectToNow(Element* dst, std::vector<Element*>& elements) {
      int did = dst->id;
      int n = currentState->left[id];
      int m = dst->canReceivePacketNumberNow();
      if (m < n) n = m;
      if (n > 0) {
        D(6) fprintf(stderr, "connect %d -> %d with %d packets\n", id, did, n);
        currentState->connected[id][did] += n;
        currentState->left[id] -= n;
        currentState->amounts[did] += n;
        currentState->left[did] += n;
        n += dst->onConnectedNow(this, elements);
      }
      return n;
    }
  };

  struct ReceiverElement : Element {
    bool isReceiver() { return true; }
    void readXML(const char * s) { color = extractElementGroup(s, "elementGroup"); target = extractInt(s, "target"); amount = 0; }
  };
  struct TransmitterElement : Element {
    bool isSender() { return true; }
    void readXML(const char * s) { color = extractElementGroup(s, "elementGroup"); amount = extractInt(s, "amount"); target = 0; }
    bool isFulfilled() { return true; }
  };
  struct TransceiverElement : Element {
    bool isSender() { return true; }
    bool isReceiver() { return true; }
    void readXML(const char * s) { color = extractElementGroup(s, "elementGroup"); amount = extractInt(s, "amount"); target = extractInt(s, "target"); }
  };
  struct RadialTransmitterElement : Element {
    real radius;
    std::vector<int> adjIds;
    bool isSender() { return false; } // not a typical sender, cannot manually connect from this to others.
    bool isReceiver() { return true; }
    void readXML(const char * s) {
      color = extractElementGroup(s, "elementGroup");
      radius = extractReal(s, "minRadius");
    }
    bool isFulfilled() { return true; }
    bool isWireless() { return true; }
    void init(std::vector<Element*>& elements) {
      adjIds.clear();
      for (int i = 0; i < (int)elements.size(); ++i) {
        if (elements[i]->id == id || !elements[i]->isReceiver()) continue;
        if (elements[i]->pos.distance(pos) <= radius && isColorMatch(elements[i])) {
          D(3) fprintf(stderr, "RadialTransmitterElement %d can reach %d\n", id, i);
          adjIds.push_back(i);
        }
      }
    }

    int canReceivePacketNumberNow() { return 32767; }
    bool canReceivePacketNow() { return true; }
    int onConnectedNow(Element* src, std::vector<Element*>& elements) {
      int result = 0;
      int n = elements.size();
      int npacket = 0; // currentState->connected[src->id][id];
      for (int i = 0; i < n; ++i) { npacket += currentState->connected[i][id]; }
      // connect to all elements in range
      for (int i : adjIds) {
        if (elements[i]->id == id || !elements[i]->isReceiver()) continue;
        int nCanReceive = elements[i]->canReceivePacketNumberNow();
        if (nCanReceive == 0) continue;
        int nCurrTransmit = currentState->connected[id][i];
        if (nCurrTransmit == 0) {
          // haven't connect, we can connect to it only when it does not connect to us
          if (currentState->connected[i][id] > 0) continue;
        }
        if (nCurrTransmit == npacket) continue; // have no more packets to transmit
        int nNewPacket = npacket - nCurrTransmit;
        if (nNewPacket > nCanReceive) nNewPacket = nCanReceive;
        currentState->connected[id][i] += nNewPacket;
        currentState->left[i] += nNewPacket;
        currentState->amounts[i] += nNewPacket;
        result += nNewPacket;
        result += elements[i]->onConnectedNow(this, elements);
        D(3) fprintf(stderr, "RadialTransmitterElement %d give %d new packets to %d\n", id, nNewPacket, i);
      }
      return result;
    }
  };
  struct SwapperTransmitterElement : Element {
    ElementGroup swapColor;
    bool isSender() { return true; }
    bool isReceiver() { return true; }
    bool isColorFixed() { return false; }
    void readXML(const char * s) { color = extractElementGroup(s, "swapGroup1"); swapColor = extractElementGroup(s, "swapGroup2"); amount = extractInt(s, "amount"); target = extractInt(s, "target"); }
    bool canReceiveColor(ElementGroup color) { return this->color == color || this->swapColor == color; }
    // state->colorSwapped:
    //   0: not connected, accept either color.
    //   1: accept color only. give out swapColor.
    //   2: accept swapColor only; give out color.
    bool canReceiveColorNow(ElementGroup color) {
      switch (currentState->colorSwapped[id]) {
        case 0:
          return this->color == color || this->swapColor == color;
        case 1:
          return this->color == color;
        case -1:
          return this->swapColor == color;
      }
      assert(false);
    }
    ElementGroup colorNow() {
      switch (currentState->colorSwapped[id]) {
        case -1:
          return this->color;
        case 1:
          return this->swapColor;
        default:
          assert(false);
      }
    }
    int onConnectedNow(Element* src, std::vector<Element*>& elements) {
      if (currentState->colorSwapped[id] == 0) {
        currentState->colorSwapped[id] = (src->colorNow() == this->color ? 1 : -1);
        D(3) fprintf(stderr, "SwapperTransmitter %d gets connected, colorSwapped is set to %d\n", id, currentState->colorSwapped[id]);
      }
      return 0;
    }
    bool isColorMatch(Element* dst) {
      return dst->canReceiveColor(color) || dst->canReceiveColor(swapColor);
    }
  };
  struct CellTransmitterElement : Element {
    bool isSender() { return true; }
    bool isReceiver() { return true; }
    void readXML(const char * s) { color = extractElementGroup(s, "elementGroup"); target = amount = 0; }
    bool isFulfilled() { return true; }
    int canReceivePacketNumberNow() { return 32767; }
    bool canReceivePacketNow() { return true; }

    int onConnectedNow(Element* src, std::vector<Element*>& elements) {
      syncToAllCellTransmittersNow(elements);
      return 0; 
    }
    int connectToNow(Element* dst, std::vector<Element*>& elements) {
      int result = Element::connectToNow(dst, elements);
      if (result > 0) syncToAllCellTransmittersNow(elements);
      return result;
    }
    real inBetweenRadius() { return 0.5; } // CellTransmitter is smaller than other elements
    bool isColorMatch(Element* dst) {
      if (dst->type == CellTransmitter) return false; // hack: unrelated to color, but forbid CellTransmitter self-connect.
      return dst->canReceiveColor(color);
    }
    private: void syncToAllCellTransmittersNow(std::vector<Element*>& elements) {
      int left = currentState->left[id];
      for (auto& e : elements) {
        if (e->id == id) continue;
        if (e->color == color && e->type == type) {
          currentState->amounts[e->id] = left;
          currentState->left[e->id] = left;
        }
      }
    }
  };
  struct SignalBoosterElement : Element {
    bool isSender() { return true; }
    bool isReceiver() { return true; }
    void readXML(const char * s) { color = extractElementGroup(s, "elementGroup"); target = amount = 0; }
    bool isFulfilled() { return true; }

    int canReceivePacketNumberNow() {
      // can not receive anything if is wired (amount > 0)
      if (currentState->amounts[id] > 0) return 0;
      return 32767;
    }
    int onConnectedNow(Element* src, std::vector<Element*>& elements) {
      assert(currentState->left[id] > 0);
      assert(currentState->left[id] == currentState->amounts[id]);
      currentState->left[id] *= 2;
      return currentState->amounts[id];
    }
  };


  struct BlockElement : Element {
    bool isBlock() { return true; }
    virtual bool canBlock(ElementGroup color, geometry::LineSegment& l) = 0;
  };
  struct SignalBlockElement : BlockElement {
    geometry::LineSegment l;
    void readXML(const char * s) {
      color = extractElementGroup(s, "blockGroup");
      l.x1 = extractReal(s, "sx"); l.y1 = extractReal(s, "sy");
      l.x2 = extractReal(s, "ex"); l.y2 = extractReal(s, "ey"); 
    }
    bool canBlock(ElementGroup color, geometry::LineSegment& line) {
      if (color != this->color) return false;
      return this->l.intersect(line);
    }
  };
  struct SignalBlockCircleElement : BlockElement {
    real radius;
    void readXML(const char * s) { color = extractElementGroup(s, "blockGroup"); radius = extractReal(s, "radius"); }
    bool canBlock(ElementGroup color, geometry::LineSegment& line) {
      if (color != this->color) return false;
      real d1 = pos.distance(*line.p1());
      real d2 = pos.distance(*line.p2());
      return (d1 < radius && d2 > radius) || (d1 > radius && d2 < radius) || (d1 > radius && d2 > radius && line.distance(pos) < radius);
    }
  };
  struct SignalBlockHexagonElement : SignalBlockElement {
    // HEX_POINTS = [[0,0]].concat([1..6].map (k) -> [- sin(PI * k / 3), - cos(PI * k / 3)])
    // a = if flip then 1 else 0
    // points = [0..6].map (i) -> [x + HEX_POINTS[i][1 - a] * scale, y + HEX_POINTS[i][a] * scale]
    std::vector<geometry::Point> points;
    void readXML(const char * s) {
      int flip;
      real radius;
      color = extractElementGroup(s, "blockGroup");
      radius = extractReal(s, "radius");
      flip = extractBool(s, "flip");
      real pi = atan(1) * 4;
      for (int i = 1; i <= 6; ++i) {
        real a[2] = {sin(pi * i / 3), cos(pi * i / 3)};
        real x = pos.x + radius * a[1 - flip];
        real y = pos.y + radius * a[flip];
        points.push_back(geometry::Point(x, y));
        D(5) { fprintf(stderr, "SignalBlockHexagon %d points[%d]: ", id, i - 1); points[i - 1].print(stderr); fputc('\n', stderr); }
      }
    }

    bool canBlock(ElementGroup color, geometry::LineSegment& line) {
      if (color != this->color) return false;
      for (int i = 0; i < 6; ++i) {
        geometry::LineSegment l(points[i], points[(i + 1) % 6]);
        if (l.intersect(line)) return true;
      }
      return false;
    }
  };


  struct ObjectiveElement : Element {
    virtual void print(FILE * fd = stderr) { fprintf(fd, "Objective: Win\n"); };
    virtual void apply() { };
    virtual void useIdMap(std::map<int, int>& idMap) { };
    bool isObjective() { return true; }
  };
  struct ObjectiveCrossedWiresElement : ObjectiveElement {
    void read(FILE * fp) {  }
    void readXML(const char * s) {  } 
    void print(FILE * fd = stderr) { fprintf(fd, "Objective: Do not cross wires\n"); };
    void apply() { objSelected |= ObjCrossWires; };
  };
  struct ObjectiveSignalCountElement : ObjectiveElement {
    int sigCount;
    void readXML(const char * s) { sigCount = extractInt(s, "signalTarget"); } 
    void print(FILE * fd = stderr) { fprintf(fd, "Objective: Do not use more than %d signals\n", sigCount); };
    void apply() { objSelected |= ObjSigCount; objSigCount = sigCount; };
  };
  struct ObjectiveTargetValueElement : ObjectiveElement {
    int targetValue;
    void readXML(const char * s) { targetValue = extractInt(s, "informationTarget"); } 
    void print(FILE * fd = stderr) { fprintf(fd, "Objective: Leave additional packet on target %d\n", targetValue); };
    void apply() { objSelected |= ObjTargetValue; objTargetValue = targetValue; };
    void useIdMap(std::map<int, int>& idMap) {
      assert(idMap.count(targetValue));
      targetValue = idMap[targetValue];
    };
  };

  Element* readElementFromXMLLine(const char * line) {
    if (!strstr(line, "<element ")) return nullptr;
    ElementType type;
    Point pos;
    int id;

    id = xml::extractInt(line, "id");
    type = extractElementType(line, "type");
    pos = xml::extractPoint(line, "position");
    if (id < 0 || type < 0) return nullptr;

    Element* e;
    switch (type) {
      case Transmitter:
        e = new TransmitterElement();
        break;
      case Transceiver:
        e = new TransceiverElement();
        break;
      case Receiver:
        e = new ReceiverElement();
        break;
      case ObjectiveCrossedWires:
        e = new ObjectiveCrossedWiresElement();
        break;
      case ObjectiveSignalCount:
        e = new ObjectiveSignalCountElement();
        break;
      case ObjectiveTargetValue:
        e = new ObjectiveTargetValueElement();
        break;
      case RadialTransmitter:
        e = new RadialTransmitterElement();
        break;
      case SwapperTransmitter:
        e = new SwapperTransmitterElement();
        break;
      case SignalBlock:
        e = new SignalBlockElement();
        break;
      case SignalBlockCircle:
        e = new SignalBlockCircleElement();
        break;
      case CellTransmitter:
        e = new CellTransmitterElement();
        break;
      case SignalBlockHexagon:
        e = new SignalBlockHexagonElement();
        break;
      case SignalBooster:
        e = new SignalBoosterElement();
        break;
      case PlacedSignal:
        // ignore it
        return nullptr;
      case InvalidElement:
        return nullptr;
    }
    e->id = id;
    e->type = type;
    e->pos = pos;
    e->color = InvalidColor;
    e->readXML(line);
    return e;
  }

  struct Level {
    std::vector<Element*> elements;
    std::vector<ObjectiveElement*> objectives;
    std::vector<BlockElement*> blocks;
    std::map<int, int> idMap;

    void clearAll() {
      idMap.clear();
      elements.clear();
      objectives.clear();
      blocks.clear();
    }

    void readXML(const char * xml) {
      clearAll();
      const char * p1 = xml; 
      for (;;) {
        const char * p2 = strchr(p1 + 1, '\n'); 
        std::string line;
        if (p2 == nullptr) line = p1; else line = std::string(p1, p2 - p1);
        D(4) fprintf(stderr, "XML LINE [%s]\n", line.c_str());
        Element * e = readElementFromXMLLine(line.c_str());
        if (e) {
          D(5) fprintf(stderr, "  GOT ELEMENT\n");
          if (e->isObjective()) {
            e->id = -1;
            objectives.push_back((ObjectiveElement*)e);
          } else if (e->isBlock()) {
            e->id = -1;
            blocks.push_back((BlockElement*)e);
          } else {
            int oldId = e->id;
            int newId = idMap.size();
            e->id = newId;
            idMap[oldId] = newId;
            elements.push_back(e);
          }
        }
        if (p2 == nullptr) break; else p1 = p2 + 1;
      }
      for (auto& obj: objectives) { obj->useIdMap(idMap); }
      for (auto& e: elements) { e->init(elements); }
    }
  };

  bool isWireAlwaysBlocked(int srcId, int dstId) {
    auto& elements = currentLevel->elements;
    Element * src = elements[srcId];
    Element * dst = elements[dstId];
    if (src->id == dst->id || !src->isSender() || !dst->isReceiver()) return true;
    if (!src->isColorMatch(dst)) return true;

    geometry::LineSegment l(src->pos, dst->pos);
    for (auto& e : elements) {
      if (e->id == src->id || e->id == dst->id) continue;
      if (e->isInBetween(l)) {
        D(3) fprintf(stderr, "# [%d, %d] blocked by element %d\n", src->id, dst->id, e->id);
        return true;
      }
    }
    // test against block elements
    bool skipBlockerTest = false;
    ElementGroup color = src->color;
    if (!src->isColorFixed()) {
      if (dst->isColorFixed()) color = dst->color; else skipBlockerTest = true;
    }
    if (!skipBlockerTest) for (auto& b : currentLevel->blocks) {
      if (b->canBlock(color, l)) {
        D(3) fprintf(stderr, "# [%d, %d] blocked by blocker %d\n", src->id, dst->id, b->id);
        return true;
      }
    }
    return false;
  }

  bool isWireBlockedByBlockersNow(int srcId, int dstId) {
    auto& elements = currentLevel->elements;
    Element * src = elements[srcId];
    Element * dst = elements[dstId];
    geometry::LineSegment l(src->pos, dst->pos);
    // fixed color elements are pre-tested in isWireAlwaysBlocked, should be good to go
    if (src->isColorFixed()) return false;
    for (auto& b : currentLevel->blocks) {
      if (b->canBlock(src->colorNow(), l)) {
        D(3) fprintf(stderr, "# [%d, %d] dynamically blocked by blocker %d\n", src->id, dst->id, b->id);
        return true;
      }
    }
    return false;
  }

  void calculateConnectable() {
    auto& elements = currentLevel->elements;
    int n = (int)elements.size();
    printf("connectable = [");
    for (int i = 0; i < n; ++i) {
      for (int j = 0; j < n; ++j) {
        // test i -> j
        // blocked by elements?
        bool blocked = isWireAlwaysBlocked(i, j);
        // blocked by block elements?
        if (!blocked) { printf("[%d, %d],", i, j); }
        connectable[i][j] = !blocked;
      } // for j
    } // for i
    printf("];\n");
  }

  struct StatePlus : State {
    StatePlus* prevState;
    std::pair<int, int> lastConnection;
    int depth;

    int flowOnce() {
      int result = 0;
      currentState = this;
      auto& es = currentLevel->elements;
      int n = (int)es.size();
      for (int i = 0; i < n; ++i) {
        if (left[i] == 0 || !currentLevel->elements[i]->isSender()) continue;
        for (int j = 0; j < n; ++j) {
          if (!connected[i][j]) continue;
          int newPackets = es[i]->connectToNow(es[j], currentLevel->elements);
          D(4) if (newPackets > 0) fprintf(stderr, "flowOnce: %d -> %d new %d\n", i, j, newPackets);
          result += newPackets;
        }
      }
      D(4) fprintf(stderr, "flowOnce: %d\n", result);
      return result;
    }

    std::vector<std::pair<int, int> > getAvailableConnections() {
      std::vector<std::pair<int, int> > result;
      currentState = this;
      auto& es = currentLevel->elements;
      int n = (int)es.size();
      for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
          if (i != j && es[i]->canConnectToNow(es[j])) {
            if (objSelected & ObjCrossWires) {
              if (es[i]->willCrossIfConnectNow(es[j], es)) continue;
            }
            result.push_back(std::make_pair(i, j));
          }
        }
      }
      return result;
    }

    StatePlus * addConnection(int srcId, int dstId) {
      StatePlus * result = new StatePlus();
      memcpy(result, this, sizeof(State));
      result->prevState = this;
      result->lastConnection = std::make_pair(srcId, dstId);
      result->depth = depth + 1;
      currentState = result;
      currentLevel->elements[srcId]->connectToNow(currentLevel->elements[dstId], currentLevel->elements);
      while (result->flowOnce() > 0);
      return result;
    }

    void printIndent(FILE * fd = stderr) {
      for (int i = 0; i < depth; ++i) { fputc(' ', fd); }
    }

    void print(bool indent = true, FILE * fd = stderr) {
      auto& es = currentLevel->elements;
      int n = (int)es.size();
      for (int i = 0; i < n; ++i) {
        bool b = false;
        for (int j = 0; j < n; ++j) {
          if (!connected[i][j]) continue;
          if (!b) {
            b = true;
            if (indent) printIndent(fd);
            fprintf(fd, "%d -> ", i);
          }
          fprintf(fd, "%d (%d); ", j, connected[i][j]);
        }
        if (b) fputc('\n', fd);
      }
    }

    void updateLeftNumbers() { // probably unused
      auto& es = currentLevel->elements;
      int n = (int)es.size();
      for (int i = 0; i < n; ++i) {
        int v = amounts[i];
        for (int j = 0; j < n; ++j) {
          v -= connected[i][j];
          v += connected[j][i];
        }
        left[i] = v;
      }
    }

    void printSteps() {
      for (StatePlus * st = this; st; st = st->prevState) {
        if (st->depth == 0) continue;
        printf("--- Step %d: %d -> %d ---\n", st->depth, st->lastConnection.first, st->lastConnection.second);
        st->print(false /* indent */, stdout);
      }
    }

    bool isWin() {
      for (auto & e : currentLevel->elements) {
        if (!e->isReceiver()) continue;
        if (!e->isFulfilled()) return false;
      }
      if (objSelected & ObjTargetValue) {
        return left[objTargetValue] > 0;
      } else {
        return true;
      }
    }
  };

  StatePlus getInitialState(Level& level = *currentLevel) {
    StatePlus result;
    memset(result.left, 0, sizeof(result.left));
    memset(result.amounts, 0, sizeof(result.amounts));
    memset(result.connected, 0, sizeof(result.connected));
    memset(result.colorSwapped, 0, sizeof(result.colorSwapped));
    result.prevState = nullptr;
    result.depth = 0;
    for (auto & e : level.elements) {
      result.amounts[e->id] = result.left[e->id] = e->amount;
    }
    return result;
  }

  struct pStateCompare {
    bool operator() (State* a, State* b) const {
      return *a < *b;
    }
  };

#ifdef DEBUG_STEP
  int debugSteps[][2] = {
    {3, 2},
    {6, 5},
    {5, 2},
    {2, 7},
    {7, 11},
    {11, 5},
    {2, 10},
    {10, 4},
    {1, 5},
    {7, 0},
    {1, 8},
    {4, 9},
  };
#endif


  std::set<StatePlus*, pStateCompare> visited;
  std::deque<StatePlus*> queue;
  bool search() {
    visited.clear(); // FIXME: memory leak
    queue.clear(); // FIXME: memory leak

    StatePlus initState = getInitialState();
    visited.insert(&initState);
    queue.push_back(&initState);

    while (!queue.empty()) {
      // fprintf(stderr, "queue size: %d\n", (int)queue.size());
      StatePlus* state = queue.front();
      queue.pop_front();
      D(1) state->print();
      if ((objSelected & ObjSigCount) && state->depth >= objSigCount) continue;
#ifdef DEBUG_STEP
      bool dsFound = false;
#endif
      for (auto& s : state->getAvailableConnections()) {
#ifdef DEBUG_STEP
        if (s.first != debugSteps[state->depth][0] || s.second != debugSteps[state->depth][1]) {
          continue;
        } else {
          dsFound = true;
        }
#endif
        D(2) {
          state->printIndent();
          fprintf(stderr, " - try connect %d -> %d\n", s.first, s.second);
        }
        StatePlus * nextState = state->addConnection(s.first, s.second);
        if (visited.count(nextState)) {
          delete nextState;
          continue;
        }
        if (nextState->isWin()) {
          nextState->printSteps();
          fprintf(stdout, "SOLVED\n");
          return true;
        }
        visited.insert(nextState);
        queue.push_back(nextState);
      }
#ifdef DEBUG_STEP
      if (!dsFound) {
        fprintf(stderr, "NOT FOLLOWING debugStep (%d -> %d) at depth %d!\n", debugSteps[state->depth][0], debugSteps[state->depth][1], state->depth);
      }
#endif
    }
    fprintf(stdout, "NOT SOLVED :(\n");
    return false;
  }
}


using namespace transmission;

int solveLevelXML(const char * xml, bool allObjTogether) {
  currentLevel = new Level();
  currentLevel->readXML(xml);
  calculateConnectable();

  // if we need to try to meet different kinds of objectives together
  int notSolved = 0;
  if (currentLevel->objectives.size() > 0) {
    for (int i = 0; i < (int)currentLevel->objectives.size(); ++i) {
      auto& obj = currentLevel->objectives[i];
      if (!allObjTogether) {
        puts("\n");
        objSelected = ObjAbsent;
      }
      obj->print(stdout);
      obj->apply();
      if (!allObjTogether) {
        if (!search()) notSolved++;
      }
    }
  } else {
    allObjTogether = true;
  }

  if (allObjTogether) {
    if (!search()) notSolved++;
  }

  return notSolved;
}

int main(int argc, char const *argv[]) {
  int notSolved = 0;
  for (int i = 1; i < argc; ++i) {
    FILE * fp = fopen(argv[i], "r");
    if (!fp) continue;

    if (argc > 2) printf("## %s\n", argv[i]);
    std::string s;
    for (;;) {
      char buf[20];
      size_t n = fread(buf, 1, sizeof(buf), fp);
      if (n == 0) break;
      s += std::string(buf, n);
    }
    fclose(fp);
    notSolved += solveLevelXML(s.c_str(), getenv("ALLOBJ"));
  }
  return notSolved;
}

