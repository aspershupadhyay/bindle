// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Line icons from the v2 design, drawn on a 24px grid.

const paths = {
  home: <><path d="M3 10.5 12 3l9 7.5" /><path d="M5 9.5V21h14V9.5" /><path d="M10 21v-6h4v6" /></>,
  sort: <><path d="M22 12h-6l-2 3h-4l-2-3H2" /><path d="M5.5 5 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.5-7Z" /></>,
  promises: <><path d="m3 7 2 2 4-4" /><path d="m3 17 2 2 4-4" /><path d="M13 6h8M13 12h8M13 18h8" /></>,
  people: <><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M22 21v-2a4 4 0 0 0-3-3.9M16 3.1a4 4 0 0 1 0 7.8" /></>,
  wrapUp: <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />,
  desk: <><rect x="3" y="3" width="7" height="9" rx="1.5" /><rect x="14" y="3" width="7" height="5" rx="1.5" /><rect x="14" y="12" width="7" height="9" rx="1.5" /><rect x="3" y="16" width="7" height="5" rx="1.5" /></>,
  inbox: <><rect x="2" y="4" width="20" height="16" rx="2" /><path d="m22 7-10 6L2 7" /></>,
  code: <><path d="m8 8-5 4 5 4" /><path d="m16 8 5 4-5 4" /><path d="m14 4-4 16" /></>,
  board: <><rect x="3" y="3" width="18" height="18" rx="2" /><path d="M8 7v7M12 7v4M16 7v9" /></>,
  creator: <><path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3Z" /><circle cx="12" cy="13" r="3" /></>,
  settings: <><circle cx="12" cy="12" r="3" /><path d="M12 2v3M12 19v3M4.9 4.9 7 7M17 17l2.1 2.1M2 12h3M19 12h3M4.9 19.1 7 17M17 7l2.1-2.1" /></>,
  plus: <path d="M12 5v14M5 12h14" />,
  check: <path d="M20 6 9 17l-5-5" />,
  search: <><circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" /></>,
  close: <path d="M18 6 6 18M6 6l12 12" />,
  star: <path d="m12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9Z" />,
  pen: <><path d="M12 20h9" /><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" /></>,
  lock: <><rect x="4" y="11" width="16" height="10" rx="2" /><path d="M8 11V7a4 4 0 0 1 8 0v4" /></>,
  chat: <><path d="M21 12a8 8 0 0 1-11.6 7.1L4 20l1-4.6A8 8 0 1 1 21 12Z" /><path d="M8.5 11h7M8.5 14h4" /></>,
  pr: <><circle cx="6" cy="6" r="2.5" /><circle cx="6" cy="18" r="2.5" /><circle cx="18" cy="18" r="2.5" /><path d="M6 8.5v7" /><path d="M18 15.5V9a3 3 0 0 0-3-3h-4" /><path d="m13 4-2 2 2 2" /></>,
  comment: <path d="M20 15a2 2 0 0 1-2 2H8l-4 4V5a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2Z" />,
  issue: <><circle cx="12" cy="12" r="9" /><circle cx="12" cy="12" r="1.5" /></>,
} as const;

export type IconName = keyof typeof paths;

export function Icon({ name, size = 20 }: { name: IconName; size?: number }) {
  return (
    <svg
      aria-hidden="true"
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.6}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {paths[name]}
    </svg>
  );
}
