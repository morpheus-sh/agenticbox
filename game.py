#!/usr/bin/env python3
"""
game.py — RPG progression system for AgenticBox founder contract.

Usage:
  python game.py event feature_shipped
  python game.py event customer_interview
  python game.py quest add "Publish landing page" 100 2 distribution
  python game.py quest complete "Publish landing page"
  python game.py status
  python game.py streak-check
"""

import json
import sys
from datetime import date, datetime
from pathlib import Path

GAME_FILE = Path(__file__).parent / "game.json"


def load_game():
    with open(GAME_FILE) as f:
        return json.load(f)


def save_game(data):
    with open(GAME_FILE, "w") as f:
        json.dump(data, f, indent=2)


def recalc_level(data):
    xp = data["founder"]["xp"]
    thresholds = data["xp_thresholds"]
    level = 1
    for i, threshold in enumerate(thresholds):
        if xp >= threshold:
            level = i + 1
    data["founder"]["level"] = level


def check_achievements(data):
    earned = []
    for ach in data["achievement_definitions"]:
        if ach["id"] in data["achievements"]:
            continue
        # Simple eval of condition (safe: only compares metrics)
        condition = ach["condition"]
        # Replace metric names with values
        for metric, value in data["metrics"].items():
            condition = condition.replace(metric, str(value))
        condition = condition.replace("streak_days", str(data["founder"]["streak_days"]))
        try:
            if eval(condition):
                data["achievements"].append(ach["id"])
                data["founder"]["xp"] += ach["xp_bonus"]
                earned.append(ach)
        except Exception:
            pass
    return earned


def update_streak(data):
    today = date.today().isoformat()
    last = data["founder"]["last_activity_date"]
    if last == today:
        return False  # already counted today
    if last is None:
        data["founder"]["streak_days"] = 1
    else:
        last_date = date.fromisoformat(last)
        delta = (date.today() - last_date).days
        if delta == 1:
            data["founder"]["streak_days"] += 1
        elif delta > 1:
            data["founder"]["streak_days"] = 1
    data["founder"]["last_activity_date"] = today
    return True


def cmd_event(event_name):
    data = load_game()
    rewards = data["xp_rewards"]
    if event_name not in rewards:
        print(f"Unknown event: {event_name}")
        print(f"Available: {', '.join(rewards.keys())}")
        return

    xp = rewards[event_name]
    data["founder"]["xp"] += xp

    # Update metric
    metric_map = {
        "feature_shipped": "features_shipped",
        "customer_interview": "customer_conversations",
        "content_published": "content_published",
        "commit": "commits",
        "bug_fixed": "bugs_fixed",
        "revenue_event": "revenue_events",
    }
    if event_name in metric_map:
        data["metrics"][metric_map[event_name]] += 1

    streak_updated = update_streak(data)
    earned = check_achievements(data)
    recalc_level(data)
    save_game(data)

    print(f"+{xp} XP → {data['founder']['xp']} total (Level {data['founder']['level']})")
    if streak_updated:
        print(f"🔥 Streak: {data['founder']['streak_days']} days")
    for ach in earned:
        print(f"🏆 ACHIEVEMENT UNLOCKED: {ach['name']} (+{ach['xp_bonus']} XP)")


def cmd_quest_add(title, xp, difficulty, category):
    data = load_game()
    quest = {
        "title": title,
        "xp": xp,
        "difficulty": difficulty,
        "category": category,
        "created": date.today().isoformat()
    }
    data["quests"]["active"].append(quest)
    save_game(data)
    print(f"Quest added: {title} (+{xp} XP, difficulty {difficulty}, {category})")


def cmd_quest_complete(title):
    data = load_game()
    for i, quest in enumerate(data["quests"]["active"]):
        if quest["title"] == title:
            data["founder"]["xp"] += quest["xp"]
            data["quests"]["completed"].append(quest)
            data["quests"]["active"].pop(i)
            earned = check_achievements(data)
            recalc_level(data)
            save_game(data)
            print(f"Quest complete: {title} (+{quest['xp']} XP)")
            for ach in earned:
                print(f"🏆 ACHIEVEMENT UNLOCKED: {ach['name']} (+{ach['xp_bonus']} XP)")
            return
    print(f"Quest not found: {title}")


def cmd_status():
    data = load_game()
    f = data["founder"]
    m = data["metrics"]
    print(f"=== Founder: Level {f['level']} | {f['xp']} XP | Streak: {f['streak_days']} days ===")
    print(f"Next level: {data['xp_thresholds'][f['level']] if f['level'] < len(data['xp_thresholds']) else 'MAX'} XP")
    print()
    print("Metrics:")
    for k, v in m.items():
        print(f"  {k}: {v}")
    print()
    print(f"Active quests ({len(data['quests']['active'])}):")
    for q in data["quests"]["active"]:
        print(f"  • {q['title']} (+{q['xp']} XP, diff {q['difficulty']}, {q['category']})")
    print()
    print(f"Completed quests: {len(data['quests']['completed'])}")
    print(f"Achievements: {len(data['achievements'])}")
    for ach_id in data["achievements"]:
        ach = next(a for a in data["achievement_definitions"] if a["id"] == ach_id)
        print(f"  🏆 {ach['name']}")


def cmd_streak_check():
    data = load_game()
    update_streak(data)
    save_game(data)
    print(f"Streak: {data['founder']['streak_days']} days (last: {data['founder']['last_activity_date']})")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return

    cmd = sys.argv[1]

    if cmd == "event":
        if len(sys.argv) < 3:
            print("Usage: game.py event <event_name>")
            return
        cmd_event(sys.argv[2])

    elif cmd == "quest":
        sub = sys.argv[2] if len(sys.argv) > 2 else "help"
        if sub == "add":
            if len(sys.argv) < 6:
                print("Usage: game.py quest add <title> <xp> <difficulty> <category>")
                return
            cmd_quest_add(sys.argv[3], int(sys.argv[4]), int(sys.argv[5]), sys.argv[6])
        elif sub == "complete":
            if len(sys.argv) < 4:
                print("Usage: game.py quest complete <title>")
                return
            cmd_quest_complete(" ".join(sys.argv[3:]))
        else:
            print("Usage: game.py quest add|complete ...")

    elif cmd == "status":
        cmd_status()

    elif cmd == "streak-check":
        cmd_streak_check()

    else:
        print(f"Unknown command: {cmd}")
        print(__doc__)


if __name__ == "__main__":
    main()