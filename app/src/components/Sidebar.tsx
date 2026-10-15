// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The left rail: logo, the Life and Work switch, the screens of the current
// mode, spaces, and Jot down (v2 design, Main and Desk).

import { useEffect, useState } from "react";
import { Avatar, AvatarPair } from "./Avatar";
import { Logo } from "./Logo";
import { useSettings } from "../lib/settings";
import { commands, type Mode, type Space } from "../lib/commands";
import { useSources } from "../lib/sources";
import { screens, settingsScreen, type ScreenId } from "../screens";
import { Icon } from "./Icon";

interface SidebarProps {
  mode: Mode;
  current: ScreenId;
  unsorted: number;
  /** Reviews people are waiting on, from GitHub. */
  reviewsOwed: number;
  /** Emails waiting on your reply, from Gmail. */
  repliesOwed: number;
  /** YouTube comments waiting on a reply. */
  commentsWaiting: number;
  /** Shared spaces on this device, and the one open now. */
  spaces: Space[];
  currentSpace: string | null;
  onSpace: (id: string) => void;
  onMode: (mode: Mode) => void;
  onScreen: (id: ScreenId) => void;
  onJotDown: () => void;
}

export function Sidebar({ mode, current, unsorted, reviewsOwed, repliesOwed, commentsWaiting, spaces, currentSpace, onSpace, onMode, onScreen, onJotDown }: SidebarProps) {
  const { settings } = useSettings();
  return (
    <aside className="sidebar" aria-label="Bindle navigation">
      <div className="brand">
        <Logo size={48} alt="Bindle logo, a cloth bag tied to a stick" />
        <div>
          <div className="brand-name">Bindle</div>
          <div className="brand-line">Carry less in your head</div>
        </div>
      </div>

      <div className="mode-switch" role="group" aria-label="Mode" data-tour="mode">
        {(["life", "work"] as const).map((m) => (
          <button key={m} type="button" aria-pressed={mode === m} onClick={() => onMode(m)}>
            {m === "life" ? "Life" : "Work"}
          </button>
        ))}
      </div>

      <nav className="nav" aria-label={mode === "life" ? "Life" : "Work"}>
        {screens[mode].map((screen) => (
          <button
            key={screen.id}
            type="button"
            className="nav-item"
            data-tour={`nav-${screen.id}`}
            aria-current={current === screen.id ? "page" : undefined}
            onClick={() => onScreen(screen.id)}
          >
            <Icon name={screen.icon} />
            {screen.label}
            {screen.id === "sort" && unsorted > 0 && <span className="badge">{unsorted} new</span>}
            {screen.id === "inbox" && repliesOwed > 0 && (
              <span className="badge" aria-label={`${repliesOwed} ${repliesOwed === 1 ? "email" : "emails"} waiting on your reply`}>
                {repliesOwed}
              </span>
            )}
            {screen.id === "creator" && commentsWaiting > 0 && (
              <span className="badge" aria-label={`${commentsWaiting} ${commentsWaiting === 1 ? "comment" : "comments"} waiting on your reply`}>
                {commentsWaiting}
              </span>
            )}
            {screen.id === "code" && reviewsOwed > 0 && (
              <span className="badge" aria-label={`${reviewsOwed} ${reviewsOwed === 1 ? "review" : "reviews"} waiting on you`}>
                {reviewsOwed}
              </span>
            )}
          </button>
        ))}
      </nav>

      {mode === "work" ? (
        <Sources current={current} onScreen={onScreen} />
      ) : (
        <div className="spaces">
          <div className="eyebrow">Spaces</div>
          <div className="space">
            <Avatar name={settings.name ?? "Me"} avatar={settings.avatar} size={32} />
            Mine<span className="space-note">private</span>
          </div>
          {spaces.map((space) => (
            <button
              key={space.id}
              type="button"
              className="nav-item space"
              data-tour="space-ours"
              aria-current={current === "ours" && currentSpace === space.id ? "page" : undefined}
              onClick={() => onSpace(space.id)}
            >
              <SpaceFaces spaceId={space.id} />
              {space.name}
              <span className="space-note">
                <span className={`dot ${space.waiting > 0 ? "honey" : "sage"}`} aria-hidden="true" />
                {space.waiting > 0 ? "to sync" : "synced"}
              </span>
            </button>
          ))}
        </div>
      )}

      <div className="sidebar-foot">
        <button type="button" className="button-primary jot-button" data-tour="jot" onClick={onJotDown}>
          <Icon name="plus" size={18} />
          Jot down
        </button>
        <div className="shortcut">
          <kbd>Alt</kbd><kbd>Shift</kbd><kbd>J</kbd> anywhere
        </div>
        <button
          type="button"
          className="nav-item quiet"
          aria-current={current === settingsScreen.id ? "page" : undefined}
          onClick={() => onScreen(settingsScreen.id)}
        >
          <Icon name="settings" size={18} />
          Settings
        </button>
      </div>
    </aside>
  );
}

/** Work's sources, each with how it is doing, as in the v2 sidebar. */
function Sources({ current, onScreen }: { current: ScreenId; onScreen: (id: ScreenId) => void }) {
  const sources = useSources();
  return (
    <div className="spaces sources">
      <div className="eyebrow">Sources</div>
      {sources.map((source) => (
        <button
          key={source.screen}
          type="button"
          className="nav-item space source"
          aria-current={current === source.screen ? "true" : undefined}
          onClick={() => onScreen(source.screen)}
        >
          <span className={`dot ${source.state === "ok" ? "sage" : source.state === "error" ? "honey" : "off"}`} aria-hidden="true" />
          {source.name}
          <span className={`space-note ${source.state === "error" ? "warn" : ""}`}>{source.note}</span>
        </button>
      ))}
    </div>
  );
}

/** Everyone who carries a space, side by side. */
function SpaceFaces({ spaceId }: { spaceId: string }) {
  const [names, setNames] = useState<string[]>([]);
  useEffect(() => {
    commands.oursView(spaceId).then(
      (view) => setNames(view.members.map((m) => m.name)),
      () => setNames([]),
    );
  }, [spaceId]);
  return <AvatarPair people={names.map((name) => ({ name, avatar: null }))} size={32} />;
}
