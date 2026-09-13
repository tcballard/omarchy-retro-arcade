#pragma once
// Independent artwork route fixtures. Each polyline must pass an actual
// radius-offset engine collision sweep; no pathfinding runs in the game.
namespace CircuitRouteFixtures {
struct Route { const char* name; std::vector<vector2> points; };
inline const std::vector<Route>& Routes() {
 static const std::vector<Route> routes={
  {"shooter", {{916,636},{860,304},{850,270},{816,198},{778,152},{680,80}}},
  {"rear orbit", {{680,80},{610,68},{450,60}}},
  {"underpass", {{450,60},{370,94},{370,150}}},
  {"left upper field", {{370,150},{380,196},{432,268},{540,390}}},
  {"right target exit", {{624,110},{624,148},{614,156},{550,164},{550,170}}},
  {"upper right field", {{660,170},{676,220},{676,276}}},
  {"lower bumpers", {{530,204},{522,246},{476,316}}},
  {"left loop", {{216,536},{286,404},{284,370},{274,336},{264,320}}},
  {"left inlane", {{244,710},{282,770},{344,844}}},
  {"left outlane", {{164,650},{124,976}}},
  {"right inlane", {{810,680},{802,736},{736,824}}},
  {"right outlane", {{860,676},{886,870},{886,930}}},
  {"module back", {{816,324},{820,436}}},
  {"module front", {{700,310},{706,418},{710,440}}},
  {"left sling back", {{244,600},{250,736}}},
  {"right sling back", {{816,590},{818,736}}},
  {"main field", {{540,400},{536,924}}},
  {"drain", {{536,940},{536,1010}}},
 }; return routes;
}
}
