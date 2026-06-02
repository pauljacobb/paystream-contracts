// SPDX-License-Identifier: Apache-2.0
import React, { useState } from "react";

const STEPS = [
  { label: "Connect Wallet", icon: "🔗" },
  { label: "Fund Account", icon: "💰" },
  { label: "Configure Stream", icon: "⚙️" },
  { label: "Confirm", icon: "✅" },
];

const STEP_CONTENT: React.FC<{ step: number }>[] = [
  () => (
    <div className="wizard-step-body">
      <h3>Step 1: Connect Your Wallet</h3>
      <p>Install <a href="https://freighter.app" target="_blank" rel="noreferrer">Freighter</a> and connect it to PayStream. Your wallet is your identity on the Stellar network — no sign-up required.</p>
      <ul className="wizard-checklist">
        <li>Install the Freighter browser extension</li>
        <li>Create or import a Stellar account</li>
        <li>Click <strong>Connect Freighter</strong> on the main page</li>
      </ul>
    </div>
  ),
  () => (
    <div className="wizard-step-body">
      <h3>Step 2: Fund Your Account</h3>
      <p>Your account needs tokens to create a stream. On testnet you can use Friendbot to get free XLM.</p>
      <ul className="wizard-checklist">
        <li>Ensure your account has enough XLM for the deposit</li>
        <li>On testnet: use <a href="https://laboratory.stellar.org/#account-creator?network=test" target="_blank" rel="noreferrer">Stellar Friendbot</a></li>
        <li>On mainnet: fund via an exchange or wallet transfer</li>
      </ul>
    </div>
  ),
  () => (
    <div className="wizard-step-body">
      <h3>Step 3: Configure Your Stream</h3>
      <p>Set up the stream parameters for your employee.</p>
      <ul className="wizard-checklist">
        <li>Enter the employee's Stellar public key</li>
        <li>Choose the token (e.g., USDC)</li>
        <li>Set the deposit amount and rate per second</li>
        <li>Optionally set a stop time</li>
      </ul>
    </div>
  ),
  () => (
    <div className="wizard-step-body">
      <h3>Step 4: Confirm &amp; Launch</h3>
      <p>Review your stream details and submit the transaction. Once confirmed on-chain, your employee starts earning immediately.</p>
      <ul className="wizard-checklist">
        <li>Double-check the employee address</li>
        <li>Verify the deposit and rate</li>
        <li>Approve the transaction in Freighter</li>
        <li>Your stream is live! 🎉</li>
      </ul>
    </div>
  ),
];

const STORAGE_KEY = "paystream_onboarding_done";

interface OnboardingWizardProps {
  onComplete: () => void;
}

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const [step, setStep] = useState(0);

  const isLast = step === STEPS.length - 1;

  const handleNext = () => {
    if (isLast) {
      localStorage.setItem(STORAGE_KEY, "1");
      onComplete();
    } else {
      setStep((s) => s + 1);
    }
  };

  const handleSkip = () => {
    localStorage.setItem(STORAGE_KEY, "1");
    onComplete();
  };

  const StepBody = STEP_CONTENT[step];

  return (
    <div className="wizard-overlay" role="dialog" aria-modal="true" aria-label="Onboarding wizard">
      <div className="wizard-card card">
        {/* Progress indicator */}
        <div className="wizard-progress" role="list" aria-label="Wizard steps">
          {STEPS.map((s, i) => (
            <div
              key={i}
              className={`wizard-step-dot${i === step ? " wizard-step-dot--active" : i < step ? " wizard-step-dot--done" : ""}`}
              role="listitem"
              aria-current={i === step ? "step" : undefined}
              aria-label={`${s.label}${i < step ? " (completed)" : ""}`}
            >
              <span className="wizard-dot-icon" aria-hidden="true">
                {i < step ? "✓" : s.icon}
              </span>
              <span className="wizard-dot-label">{s.label}</span>
            </div>
          ))}
        </div>

        {/* Step content */}
        <StepBody step={step} />

        {/* Navigation */}
        <div className="wizard-nav">
          <button
            className="btn btn-secondary"
            onClick={() => setStep((s) => s - 1)}
            disabled={step === 0}
            aria-label="Go to previous step"
          >
            ← Back
          </button>
          <button className="btn btn-secondary wizard-skip" onClick={handleSkip}>
            Skip
          </button>
          <button className="btn" onClick={handleNext} aria-label={isLast ? "Finish onboarding" : "Go to next step"}>
            {isLast ? "Get Started →" : "Next →"}
          </button>
        </div>
      </div>
    </div>
  );
}

/** Returns true if the user has not yet completed onboarding. */
export function shouldShowOnboarding(): boolean {
  return !localStorage.getItem(STORAGE_KEY);
}
