// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// First run (handbook P9): four short steps, each skippable, so a new person
// reaches Home in well under three minutes. Reminders start quiet; only the
// ones the person ticks are turned on.

import { useEffect, useRef, useState, type FormEvent } from "react";
import { Logo } from "../components/Logo";
import hills from "../art/hills.png";
import { JotField } from "../components/JotField";
import { useSettings } from "../lib/settings";

type Step = "welcome" | "name" | "evening" | "jot";
const steps: Step[] = ["welcome", "name", "evening", "jot"];

export function FirstRun() {
  const { settings, update } = useSettings();
  const [step, setStep] = useState<Step>("welcome");
  const [name, setName] = useState(settings.name ?? "");
  const [wrapUpTime, setWrapUpTime] = useState(settings.wrap_up_time.slice(0, 5));
  const [wrapUpReminder, setWrapUpReminder] = useState(false);
  const [timedReminders, setTimedReminders] = useState(settings.timed_reminders);
  const [saved, setSaved] = useState(false);
  const jotField = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (step === "jot") jotField.current?.focus();
  }, [step]);

  const next = () => setStep(steps[steps.indexOf(step) + 1]);

  async function finish() {
    await update({
      name: name.trim() || null,
      wrap_up_time: `${wrapUpTime || "21:00"}:00`,
      wrap_up_reminder: wrapUpReminder,
      timed_reminders: timedReminders,
      onboarded: true,
    });
  }

  function submitName(event: FormEvent) {
    event.preventDefault();
    next();
  }

  return (
    <div className="first-run">
      <div className="card first-card">
        <ol className="first-dots" aria-label={`Step ${steps.indexOf(step) + 1} of ${steps.length}`}>
          {steps.map((s) => (
            <li key={s} className={s === step ? "now" : undefined} />
          ))}
        </ol>

        {step === "welcome" && (
          <>
            <Logo size={96} alt="Bindle logo, a cloth bag tied to a stick" />
            <h1 className="display">Bindle</h1>
            <p className="lead">Everything that needs you, in one bag.</p>
            <p>
              Bindle remembers the promises you make and the ones made to you, so your head does not have to.
              Everything stays on this device.
            </p>
            <button type="button" className="button-primary" onClick={next} autoFocus>
              Start
            </button>
          </>
        )}

        {step === "name" && (
          <form onSubmit={submitName} className="first-form">
            <h1 className="display">What should Bindle call you?</h1>
            <label className="field">
              <span className="visually-hidden">Your name</span>
              <input className="title-input" value={name} onChange={(e) => setName(e.target.value)} placeholder="Your first name" autoFocus />
            </label>
            <div className="first-actions">
              <button type="submit" className="button-primary">{name.trim() ? "Next" : "Skip"}</button>
            </div>
          </form>
        )}

        {step === "evening" && (
          <div className="first-form">
            <h1 className="display">Your evenings</h1>
            <p>Once a day, take about a minute to wrap up: see what you kept and decide about the rest.</p>
            <label className="field">
              <span className="field-label">I usually wind down around</span>
              <input type="time" value={wrapUpTime} onChange={(e) => setWrapUpTime(e.target.value)} />
            </label>
            <label className="check">
              <input type="checkbox" checked={wrapUpReminder} onChange={(e) => setWrapUpReminder(e.target.checked)} />
              Remind me to wrap up
            </label>
            <label className="check">
              <input type="checkbox" checked={timedReminders} onChange={(e) => setTimedReminders(e.target.checked)} />
              Remind me at the times I give promises
            </label>
            <p className="muted small">Bindle is quiet by default: at most three reminders a day, and none after 10 PM.</p>
            <div className="first-actions">
              <button type="button" className="button-primary" onClick={next} autoFocus>Next</button>
            </div>
          </div>
        )}

        {step === "jot" && (
          <div className="first-form">
            <h1 className="display">Try it</h1>
            <p>Jot down one thing you promised someone, or something someone promised you.</p>
            <JotField ref={jotField} onSaved={() => setSaved(true)} />
            {saved && <p className="muted small">It waits in Sort until you decide what it is.</p>}
            <p className="muted small">
              Anywhere on your computer, <kbd>Alt</kbd> <kbd>Shift</kbd> <kbd>J</kbd> opens Jot down.
            </p>
            <div className="first-actions">
              <button type="button" className="button-primary" onClick={() => void finish()}>
                {saved ? "Go to Home" : "Skip for now"}
              </button>
            </div>
          </div>
        )}
      </div>
      <img className="sprite first-hills" src={hills} alt="" width={384} height={160} />
    </div>
  );
}
