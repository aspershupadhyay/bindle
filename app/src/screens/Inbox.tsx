// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Work, Inbox (v2 design, Inbox; handbook P14): the emails waiting on your
// reply, your questions waiting on others, and promises you wrote in an
// email, read from Gmail. Any of them can become a promise in Sort. Bindle
// only reads: it never marks, moves or sends anything.

import { useCallback, useEffect, useState, type FormEvent } from "react";
import dakghar from "../art/dakghar.png";
import { useNotify } from "../components/Undo";
import { whenLabel } from "../lib/chips";
import { commands, errorMessage, onMailChanged, onPromisesChanged, type InboxView, type MailItem, type MailProvider } from "../lib/commands";
import { age, inboxSummary, lookedAgo, mailName } from "../lib/work";

/** "By Fri" reads "by Fri" mid-sentence. */
const lowerFirst = (words: string) => words.charAt(0).toLowerCase() + words.slice(1);

const kindWords: Record<MailItem["kind"], string> = {
  reply: "Needs reply",
  waiting: "Waiting on them",
  promise: "Your promise",
};

export function Inbox({ onOpenSort }: { onOpenSort: () => void }) {
  const [view, setView] = useState<InboxView | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const [looking, setLooking] = useState(false);
  const [hiddenNow, setHiddenNow] = useState<MailItem | null>(null);
  const notify = useNotify();

  const refresh = useCallback(() => {
    commands.inboxView().then(setView, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    const stopMail = onMailChanged(refresh);
    const stopPromises = onPromisesChanged(refresh);
    return () => {
      stopMail();
      stopPromises();
    };
  }, [refresh]);

  async function lookNow() {
    setLooking(true);
    try {
      await commands.gmailRefresh();
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setLooking(false);
      refresh();
    }
  }

  const now = new Date();
  const connected = Boolean(view?.account.address);
  const all = view ? [...view.reply, ...view.noticed, ...view.waiting] : [];
  const selected = all.find((m) => m.id === chosen) ?? all[0] ?? null;

  async function hide(item: MailItem) {
    try {
      await commands.mailDismiss(item.id);
      setHiddenNow(item);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  async function unhide() {
    if (!hiddenNow) return;
    await commands.mailRestore(hiddenNow.id).catch((e) => notify(errorMessage(e)));
    setHiddenNow(null);
  }

  return (
    <div className="page inbox-page">
      <header className="code-header">
        <div className="code-hello">
          <div className="muted small">Work · Inbox{view?.account.address ? ` · ${mailName(view.account)}` : ""}</div>
          <h1 className="display">Inbox</h1>
          {view && connected && <p className="lead">{inboxSummary(view)}</p>}
          {view && connected && (
            <p className="muted small code-synced" role="status">
              {lookedAgo(view.account.synced_at, now)}
              {" · "}
              <button type="button" className="link-button" disabled={looking} onClick={() => void lookNow()}>
                {looking ? "Looking…" : "Look now"}
              </button>
            </p>
          )}
        </div>
        <figure className="scene-frame">
          <img className="sprite scene" src={dakghar} alt="Pixel art: a small Indian post office, a dak ghar, with a red letter box" width={384} height={160} />
        </figure>
      </header>

      {view && !connected && <ConnectMail onConnected={refresh} />}

      {view && connected && (
        <>
          {view.account.error && (
            <p className="card code-problem" role="alert">
              {view.account.error}
            </p>
          )}
          {hiddenNow && (
            <p className="muted small" role="status">
              Hidden in Bindle: {hiddenNow.subject}.{" "}
              <button type="button" className="link-button" onClick={() => void unhide()}>
                Undo
              </button>
            </p>
          )}
          {view.finished.length > 0 && <FinishedMail items={view.finished} service={mailName(view.account)} />}
          {all.length === 0 ? (
            <p className="card empty">Nothing in your email needs you right now. Bindle looks again every 10 minutes.</p>
          ) : (
            <div className="inbox-columns">
              <div className="inbox-list" aria-label="Emails that need you">
                <MailGroup title="Needs reply" items={view.reply} selected={selected} onChoose={setChosen} now={now} />
                <MailGroup title="Promises you wrote" items={view.noticed} selected={selected} onChoose={setChosen} now={now} />
                <MailGroup title="Waiting on others" items={view.waiting} selected={selected} onChoose={setChosen} now={now} />
                {view.hidden > 0 && <p className="muted small">{view.hidden} marked Not needed.</p>}
              </div>
              {selected && <Reading item={selected} service={mailName(view.account)} now={now} onHide={() => void hide(selected)} onOpenSort={onOpenSort} />}
            </div>
          )}
          <p className="muted small code-account">
            Reading <strong>{view.account.address}</strong>. Only email from the last two weeks, and never newsletters, receipts or alerts.{" "}
            <button
              type="button"
              className="link-button"
              onClick={() =>
                commands.gmailDisconnect().then(
                  () => notify(`Disconnected from ${mailName(view.account)}. Promises you made stay.`),
                  (e) => notify(errorMessage(e)),
                )
              }
            >
              Disconnect
            </button>
          </p>
        </>
      )}
    </div>
  );
}

function MailGroup(props: { title: string; items: MailItem[]; selected: MailItem | null; onChoose: (id: string) => void; now: Date }) {
  if (props.items.length === 0) return null;
  const id = `mail-${props.title.toLowerCase().replace(/\s+/g, "-")}`;
  return (
    <section aria-labelledby={id}>
      <h2 id={id} className="mail-group-title">
        {props.title} <span className="muted">· {props.items.length}</span>
      </h2>
      <ul className="card list mail-list">
        {props.items.map((item) => (
          <li key={item.id}>
            <button
              type="button"
              className="mail-row"
              aria-current={props.selected?.id === item.id ? "true" : undefined}
              onClick={() => props.onChoose(item.id)}
            >
              <span className="mail-line">
                <strong>{item.person}</strong>
                <span className="muted small">{age(item.at, props.now)}</span>
              </span>
              <span className="mail-subject">{item.subject}</span>
              <span className="muted small mail-snippet">{item.snippet}</span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

function Reading({ item, service, now, onHide, onOpenSort }: { item: MailItem; service: string; now: Date; onHide: () => void; onOpenSort: () => void }) {
  const notify = useNotify();
  const [busy, setBusy] = useState(false);

  async function makePromise() {
    setBusy(true);
    try {
      await commands.mailMakePromise(item.id);
      notify(item.noticed ? "Added to Sort" : `Sent to Sort: ${item.subject}`);
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  const open = () => commands.openInGmail(item.url).catch((e) => notify(errorMessage(e)));
  const inSort = item.promise?.state === "unsorted";
  const promised = item.promise && item.promise.state !== "let_go";

  return (
    <article className="card mail-reading" aria-labelledby="reading-subject">
      <p className="muted small">
        {item.kind === "reply" ? "From" : "To"} <strong>{item.person}</strong> · {item.address} · {age(item.at, now)} ago · {kindWords[item.kind]}
      </p>
      <h2 id="reading-subject" className="mail-reading-subject">
        {item.subject}
      </h2>
      <p className="mail-reading-text">{item.snippet || "No text."}</p>

      {item.noticed ? (
        <div className="noticed">
          <span className="eyebrow">Bindle noticed a promise</span>
          <strong>
            {item.noticed.title}
            {item.noticed.when && `, ${lowerFirst(whenLabel(item.noticed.when, now, item.noticed.is_deadline))}`}
          </strong>
          <span className="muted small">You wrote it to {item.person}. It is not on your list yet.</span>
          <div className="update-actions">
            <button type="button" className="button-primary small-button" disabled={busy || Boolean(promised)} onClick={() => void makePromise()}>
              Add to promises
            </button>
            <button type="button" className="button-quiet small-button" onClick={onHide}>
              Not a promise
            </button>
          </div>
        </div>
      ) : (
        <div className="update-actions">
          {inSort ? (
            <button type="button" className="link-button small" onClick={onOpenSort}>
              Waiting in Sort
            </button>
          ) : promised ? (
            <span className="chip plain">A promise</span>
          ) : (
            <button type="button" className="button-primary small-button" disabled={busy} onClick={() => void makePromise()}>
              Make it a promise
            </button>
          )}
          <button type="button" className="button-quiet small-button" onClick={onHide}>
            Not needed
          </button>
          {item.url && (
            <button type="button" className="button-quiet small-button" onClick={() => void open()}>
              Open in {service}
            </button>
          )}
        </div>
      )}
      <p className="muted small">Bindle never sends email. Replies happen in {service}.</p>
    </article>
  );
}

function FinishedMail({ items, service }: { items: MailItem[]; service: string }) {
  const notify = useNotify();
  async function close(item: MailItem) {
    const promise = item.promise;
    if (!promise) return;
    try {
      if (promise.state === "unsorted") await commands.letGo(promise.id);
      else await commands.markDone(promise.id);
      notify(promise.state === "unsorted" ? `Let go: ${item.subject}` : `Done: ${item.subject}`);
    } catch (error) {
      notify(errorMessage(error));
    }
  }
  return (
    <section aria-labelledby="mail-finished" className="card code-finished">
      <h2 id="mail-finished">Answered already?</h2>
      <p className="muted small">These no longer need a reply in {service}. If the promise is kept, mark it so.</p>
      <ul className="stack">
        {items.map((item) => (
          <li key={item.id} className="finished-row">
            <span>
              <strong>{item.person}</strong>: {item.subject}
            </span>
            <button type="button" className="button-quiet small-button" onClick={() => void close(item)}>
              {item.promise?.state === "unsorted" ? "Let it go" : "Mark done"}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

function ConnectMail({ onConnected }: { onConnected: () => void }) {
  const notify = useNotify();
  const [providers, setProviders] = useState<MailProvider[]>([]);
  const [providerId, setProviderId] = useState("gmail");
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [host, setHost] = useState("");
  const [port, setPort] = useState("993");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    commands.mailProviders().then(setProviders, (e) => notify(errorMessage(e)));
  }, [notify]);
  const provider = providers.find((p) => p.id === providerId);
  const other = providerId === "other";
  const name = provider && !other ? provider.name : "your mailbox";

  async function openPasswordPage() {
    if (!provider?.password_page) return;
    try {
      await commands.openInGmail(provider.password_page);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  async function connect(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setProblem(null);
    try {
      const account = await commands.gmailConnect(address, password, providerId, other ? host : undefined, other ? Number(port) || 993 : undefined);
      setPassword("");
      notify(`Connected to ${account.address}`);
      onConnected();
    } catch (error) {
      setProblem(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="card code-connect" aria-labelledby="connect-mail">
      <h2 id="connect-mail">Connect your email</h2>
      <p className="muted">
        See who is waiting on your reply, which of your questions are still open, and the promises you made in an email. Bindle only reads. It never
        marks anything as read, moves it, or sends email.
      </p>
      <div className="pills" role="group" aria-label="Your mail service">
        {providers.map((p) => (
          <button key={p.id} type="button" className="pill" aria-pressed={providerId === p.id} onClick={() => setProviderId(p.id)}>
            {p.name}
          </button>
        ))}
      </div>
      <ol className="connect-steps">
        <li>
          <span>{provider?.note ?? "Your mail service gives each app its own password."}</span>{" "}
          {provider?.password_page && (
            <button type="button" className="button-quiet small-button" onClick={() => void openPasswordPage()}>
              Make an app password
            </button>
          )}
          <span className="muted small"> Name it "Bindle", then copy the password it shows.</span>
        </li>
        <li>
          <form className="connect-form" onSubmit={(e) => void connect(e)}>
            {other && (
              <>
                <label className="field">
                  <span className="field-label">IMAP server</span>
                  <input value={host} onChange={(e) => setHost(e.target.value)} placeholder="imap.example.com" spellCheck={false} autoCapitalize="off" />
                </label>
                <label className="field field-narrow">
                  <span className="field-label">Port</span>
                  <input inputMode="numeric" value={port} onChange={(e) => setPort(e.target.value.replace(/\D/g, ""))} placeholder="993" />
                </label>
              </>
            )}
            <label className="field">
              <span className="field-label">Your {other ? "email" : name} address</span>
              <input type="email" autoComplete="email" value={address} onChange={(e) => setAddress(e.target.value)} placeholder="you@example.com" />
            </label>
            <label className="field">
              <span className="field-label">The app password</span>
              <input type="password" autoComplete="off" spellCheck={false} value={password} onChange={(e) => setPassword(e.target.value)} placeholder="abcd efgh ijkl mnop" />
            </label>
            <button type="submit" className="button-primary small-primary" disabled={busy || !address.trim() || !password.trim() || (other && !host.trim())}>
              {busy ? "Checking…" : "Connect"}
            </button>
          </form>
        </li>
      </ol>
      {problem && (
        <p className="error-text small" role="alert">
          {problem}
        </p>
      )}
      <p className="muted small">The app password stays in this computer's keychain. It is never saved in Bindle's data, its exports or its safety copies.</p>
    </section>
  );
}
