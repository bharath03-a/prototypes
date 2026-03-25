# Clinical Chart Index — Project Specification

## What we are building

A tool that reads patient chart PDFs and automatically generates a structured outline of the document — similar to a table of contents — using Google's Gemini AI. This outline is then used to answer clinical questions accurately, by navigating to the right section of the chart rather than searching through the whole document at once.

The tool compares two different ways of organizing the outline side by side, so clinical teams can evaluate which approach produces better answers for their use case.

---

## The problem we are solving

Standard AI document tools split documents into small pieces and search for the piece that sounds most similar to your question. This works poorly for patient charts because:

- Medical questions require reasoning and context, not just keyword matching
- Splitting a chart destroys the relationships between sections (a medication in one section connects to a diagnosis in another)
- Charts are inconsistently structured — each provider and EHR system formats them differently

Our approach: ask Gemini to read the whole chart and produce a structured outline. When a question is asked, Gemini navigates that outline to find the right section and reads only those pages — the way a clinician would flip to the right part of a chart.

---

## Source documents

Patient charts are messy. The tool must handle all of the following:

- **Mixed format PDFs** — a single file may contain some pages that are scanned images (photographed or faxed) and other pages that are digitally typed. Both types appear in the same file.
- **Inconsistent structure** — some charts have clear section headings; others are dense paragraphs of free text; others are pre-printed forms. Structure varies by provider, clinic, and EHR system.
- **Variable length** — from a single visit note (5–20 pages) to a full longitudinal record spanning years (100+ pages).
- **Mixed content types** — typed clinical notes, handwritten annotations, lab result tables, referral letters, discharge summaries, consent forms.

The tool must handle all of these without any manual preprocessing. The PDF is passed directly to Gemini, which handles both typed and scanned content natively.

---

## Two approaches to outlining

We build two outlines for every chart and compare them. Both outlines contain the same information — title, page range, and a short summary for each item — but organized differently.

### Approach A — organized by section

The outline groups the chart by clinical topic. Each top-level item is a clinical section. This is how a textbook or a structured note is organized.

Example structure:

- Chief complaint (page 1)
- History of present illness (pages 1–2)
- Medications (page 3)
  - Current medications
  - Discontinued medications
- Assessment and plan (pages 4–5)
- Lab results (page 6)

Best for: single-visit charts, structured documents, questions about a specific clinical topic.

### Approach B — organized by visit

The outline groups the chart by encounter, in chronological order. Each top-level item is a visit. Clinical sections are nested inside each visit. This is how a longitudinal record is organized.

Example structure:

- Visit January 2024 — Primary care (pages 1–4)
  - Chief complaint
  - Assessment and plan
- Visit June 2024 — Cardiology (pages 5–9)
  - Stress test review
  - Plan
- Visit November 2024 — Emergency department (pages 10–15)
  - Triage note
  - Lab results
  - Discharge summary

Best for: multi-visit records, questions about trends over time, questions that require comparing information across visits.

---

## What each outline item contains

Every item in either outline — whether a section or a visit — contains:

- **Title** — a plain-language name (e.g. "Medications" or "Visit January 2024 — Cardiology")
- **Page range** — which pages in the PDF this item covers (e.g. pages 3–5)
- **Summary** — 2 to 3 sentences describing the clinically relevant content found in those pages
- **Child items** — any subsections or sub-visits nested within this item

No patient name, date of birth, or medical record number should appear in any title field.

---

## How questions are answered

When a user asks a question:

1. Gemini is shown the outline for one approach and asked which items are most relevant to the question
2. Gemini reads the actual pages from the PDF that those items cover
3. Gemini produces an answer based only on what is documented in those pages

The answer always includes:

- The answer text
- Which outline items were used to find the answer
- Which page numbers the answer comes from
- A confidence level: high, medium, or low
- A plain-language explanation of why the confidence is high or low

If the answer is not documented in the chart, the system must say so explicitly — it must never guess or fill in missing information.

---

## Technical requirements

These are decisions the development team must follow. They are not negotiable.

- **Package management**: use `uv` for all Python dependency management. Do not use pip, poetry, or conda.
- **AI library**: use `google-genai` — the current unified Google AI SDK. Do not use `google-generativeai`, which is deprecated.
- **AI model**: use `gemini-2.0-flash` as the default. Allow switching to `gemini-2.5-pro` for higher accuracy on complex charts via an environment variable.
- **PDF handling**: pass PDFs directly to Gemini as binary files. Do not convert to text, Markdown, or images beforehand. Gemini handles both typed and scanned pages natively.
- **Backend**: FastAPI with a single `uvicorn` server.
- **Frontend**: React 18 with TypeScript, Vite, and Tailwind CSS v4.
- **Graph visualization**: React Flow with dagre layout.

---

## Data privacy and security

- Development and testing must use de-identified sample charts only — no real patient data.
- For any deployment involving real patient data, the system must run on Google Cloud Vertex AI within a project covered by a Business Associate Agreement (BAA). The standard Gemini API (Google AI Studio) is not HIPAA-eligible.
- Nothing that identifies a patient should be written to logs, stored in files, or surfaced in the UI.
- Outline items, answers, and ratings should be stored without any chart content — only the question text, approach used, and rating score.

---

## The application — four screens

### Screen 1 — Upload

The starting point. A user drops or selects a patient chart PDF. The system builds both outlines simultaneously and shows progress. When complete, the user is taken automatically to the Explore screen.

User-facing copy (use exactly this language):

- Drop zone label: "Drop a patient chart here, or click to browse"
- Action button: "Build chart outline"
- While processing: "Reading chart…" then "Mapping sections…" and "Mapping visits…"
- On completion: "Ready — found 14 sections and 3 visits"
- On error: "Something went wrong reading this chart. Try a different file." with a Retry button.

### Screen 2 — Explore

Shows what was extracted from the chart. Both approaches are shown side by side. Users can switch between three views of the same data:

**Outline view** — a collapsible tree. Clicking an item expands its children and shows its summary. Each item shows its page range as a small label. A link on each item pre-fills the Compare screen with a question about that item.

**Contents view** — a flat list with page numbers, indented by depth. Like the table of contents in a book. Clicking a row highlights the corresponding item in the Outline view.

**Map view** — a visual graph showing the structure of the outline as a diagram. Items are boxes connected by lines. The whole graph can be panned and zoomed. Clicking a box shows the item's summary in a side panel.

When any item is clicked in any view, a side panel slides in showing:

- The item title
- The page range
- The full summary
- How many child items it contains
- A "Ask about this" button

### Screen 3 — Compare

The core evaluation experience. A user asks a question and sees both approaches answer it simultaneously, side by side.

Layout from top to bottom:

- A text input field for the question, with an "Ask" button
- A row of preset question pills (horizontally scrollable)
- Two answer panels side by side — one for each approach
- Inside each panel: the answer, which outline items were used, which pages were cited, and a confidence indicator
- Thumbs up / thumbs down buttons below each answer
- A row at the bottom: "Which gave a better answer?" with three buttons — Section approach, Visit approach, Same

Both panels load at the same time. A skeleton placeholder is shown while answers are being generated.

### Screen 4 — Questions

A catalog of pre-written evaluation questions organized into four categories. This screen helps users understand what the system is designed to do and gives evaluators a starting point.

Each question card shows:

- The question text
- A badge indicating whether it is better suited to the section approach, the visit approach, or both
- When expanded: why this question was chosen and what a good answer looks like
- A "Try this question" button that navigates to the Compare screen with the question pre-filled

---

## Pre-written evaluation questions

These are built into the application and shown on the Questions screen and as pills on the Compare screen.

### Single-fact lookup

These test whether the system can find a specific piece of information reliably.

**What medications is the patient currently taking?**
Better for: both approaches
Why this question: Tests basic information retrieval. The section approach finds the Medications section directly. The visit approach finds the same information via the most recent visit.
What a good answer looks like: A list with drug names and dosages. The section approach will likely be more concise; the visit approach may add context about when medications were started or changed.

**What are the patient's known allergies?**
Better for: section approach
Why this question: Allergies is a discrete clinical section. The section approach navigates to it directly without needing to know visit history.
What a good answer looks like: Drug or food allergies listed with the type of reaction. If not documented, the answer should say "Not documented in this chart."

**What was the patient's blood pressure at the most recent visit?**
Better for: visit approach
Why this question: Requires knowing which visit is the most recent. The visit approach organizes encounters chronologically, making this unambiguous.
What a good answer looks like: A specific numeric reading (e.g. 148/92) with the date of the visit it came from.

**What is the patient's primary diagnosis?**
Better for: both approaches
Why this question: Both approaches should find the assessment and plan. The visit approach may be better at identifying the most recent diagnosis.
What a good answer looks like: A named diagnosis, with an ICD code if one is present in the chart.

---

### Questions about trends over time

These test whether the system can compare information across multiple visits. The visit approach should perform significantly better here.

**How has the patient's blood pressure changed over time?**
Better for: visit approach
Why this question: The definitive test for longitudinal reasoning. The visit approach has all encounters as dated top-level items and can pull readings from each. The section approach has no temporal structure to compare across.
What a good answer looks like: A trend with at least two readings and their dates. The visit approach should clearly outperform the section approach on this question.

**Was the medication prescribed at the first visit still being taken at the most recent visit?**
Better for: visit approach
Why this question: Requires comparing medication lists across two specific visits. Only possible with a visit-structured outline.
What a good answer looks like: A clear yes or no, with reasoning citing both the first visit and the most recent visit by name.

**What new diagnoses were added between visits?**
Better for: visit approach
Why this question: Requires comparing clinical information across encounters. The section approach cannot do this because it has no concept of when information was recorded.
What a good answer looks like: Named diagnoses with the visit where they first appeared.

**How many times has this patient been seen, and for what reasons?**
Better for: visit approach
Why this question: A direct test of whether the visit approach correctly identifies and counts encounters.
What a good answer looks like: A count with a brief reason for each visit.

---

### Structured information extraction

These test whether the system can pull organized data out of the chart.

**List all medications with dosages.**
Better for: both approaches
Why this question: Both approaches should find medication information. The visit approach may produce a more complete list by looking across multiple visits.
What a good answer looks like: A structured list with drug name, dose, frequency, and route where present.

**What follow-up actions were ordered?**
Better for: visit approach
Why this question: Follow-up orders appear in the assessment and plan across multiple visits. The visit approach can pull all of them in chronological order.
What a good answer looks like: A list of ordered actions with the visit date each was ordered.

**What lab results are documented and what do they indicate?**
Better for: both approaches
Why this question: Tests whether the system finds lab sections and interprets values in clinical context.
What a good answer looks like: Lab test names, values, reference ranges if present, and a brief clinical interpretation.

---

### Stress tests

These test honesty and precision. The system must not guess or fill in missing information.

**Is there any documentation of a fall risk assessment?**
Better for: both approaches
Why this question: Tests whether the system admits when information is absent rather than hallucinating a plausible answer.
What a good answer looks like: "Not documented in this chart" with a note of which pages were checked. Never an invented assessment.

**Are there any contradictions in the medication list between visits?**
Better for: visit approach
Why this question: Requires comparing medication information across encounters. The answer should identify a specific discrepancy or explicitly confirm consistency — not hedge.
What a good answer looks like: A specific contradiction with the visits it appears between and the page numbers, or "No contradictions found across the visits reviewed."

**Was informed consent documented?**
Better for: both approaches
Why this question: Tests specificity and honesty. The system must look and clearly report what it finds — not infer.
What a good answer looks like: A direct yes or no with a page citation, or "Not found in this chart."

---

## Visual design requirements

### Principles

**Plain language everywhere.** The interface must never use technical terms. Words and phrases that must not appear in the user interface:

| Do not use                          | Use instead             |
| ----------------------------------- | ----------------------- |
| Node                                | Section, visit, item    |
| Strategy                            | Approach, view          |
| Vector / embedding / RAG / chunking | (never surface these)   |
| Confidence score                    | How sure we are         |
| Index                               | Chart outline           |
| Token                               | (never surface this)    |
| Tree                                | Outline, structure, map |

**Immediate response.** Every tap or click must produce a visible change within 100 milliseconds. Use placeholder skeletons while content is loading — never a blank panel.

**Side by side is the product.** The comparison layout is the most important screen. It must always be instantly clear which panel belongs to which approach. Consistent color coding must be used everywhere the two approaches appear.

**Show less, reveal more.** Default to summaries. Show detail only when the user asks for it. Do not surface page numbers, IDs, or raw data unless the user has clicked to see them.

**Accessible to everyone.** Every interactive element must work with a keyboard alone. Color is never the only way to distinguish information — every color-coded element must also have a text label. Tap targets must be large enough to use on a touchscreen (minimum 44 × 44 pixels).

### Color coding

Two colors are used consistently across the entire application to distinguish the two approaches. These colors appear on badges, panel headers, graph nodes, tree dots, TOC indicators, and answer cards.

- **Section approach** — teal (a calm, clinical green)
- **Visit approach** — purple (a distinct, readable violet)

Confidence levels use three colors:

- High confidence — green
- Medium confidence — amber
- Low confidence — red

### Typography

Two fonts are used:

- **DM Sans** — headings, navigation, screen titles
- **Inter** — all body text, labels, inputs, and buttons

Text sizes range from 11px for small badges up to 28px for the main headline on the upload screen. Body text is 14px. No bold weights heavier than medium (500) are used anywhere.

### Layout

- Maximum content width: 1280 pixels, centered on the page
- All comparison screens use a strict 50/50 two-column layout
- Panels have 16px of internal padding
- Cards and panels use 8px corner rounding; pills and badges use 20px corner rounding
- All borders are 0.5 pixels (hairline)

---

## Graph map — how it works

The Map view on the Explore screen renders the outline as a visual diagram using React Flow.

Each item in the outline becomes a box. Lines connect parent items to their children. The root item (the chart itself) sits at the top center. All other items flow downward in a tree layout, automatically positioned so boxes do not overlap.

Section approach boxes use teal styling. Visit approach boxes use purple styling. The root box uses a neutral gray.

Clicking any box opens the item detail panel on the right — showing the title, page range, summary, and an "Ask about this" button.

The graph can be zoomed and panned freely. Buttons are provided to zoom in, zoom out, and fit the entire outline onto the screen. A minimap in the corner helps navigate large charts.

---

## What success looks like

The system is working well when:

- Both outlines are generated in under 45 seconds for a 20-page chart
- The section approach correctly identifies at least 80% of the major clinical sections present in a chart
- The visit approach correctly identifies encounter boundaries when dates are visible in the document
- Answers cite the correct pages for straightforward factual questions
- A clinical reviewer rates answers as accurate for at least 4 out of 5 single-fact questions
- The visit approach produces better answers than the section approach for trend-based questions
- Both approaches perform similarly for single-fact lookup questions
- The graph map renders cleanly without overlapping boxes for charts with up to 50 items
- No patient identifiers appear in any outline item title
