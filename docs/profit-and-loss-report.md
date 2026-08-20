# Profit and loss report

## Problem

The ledger holds journal entries; nobody can read performance out of it. A
consumer wanting to know what was earned and spent over a period has to fetch
every entry and total it themselves, and would have to know which accounts are
revenue, which are costs, and which sides of the ledger they sit on.

This feature answers that in one request: an income statement for a date range,
already grouped and totalled, in a shape a client can render straight into a
report without arithmetic of its own.

Out of scope for v0: comparative periods, budgets, drill-down to entries,
currencies other than GBP, asynchronous generation.

## Scope

- Generation is synchronous. The request may be slow; the client waits.
- Amounts are minor units (pence) as integers. No currency is named anywhere —
  GBP is implicit and currency support comes later.
- Only revenue and expense accounts appear. Asset, liability, and equity
  accounts belong to a balance sheet.

## Implementation

### Report shape

Four headings, each mapped from account subclasses, each holding subheadings
that hold accounts:

| Heading            | Subheadings                     | From                                            |
| ------------------ | ------------------------------- | ----------------------------------------------- |
| Trading Income     | Sales                           | `Revenue(Sales)`                                |
| Cost of Sales      | Direct Costs                    | `Expense(DirectCosts)`                          |
| Other Income       | Other Income                    | `Revenue(OtherIncome)`                          |
| Operating Expenses | Depreciation, General, Overhead | `Expense(Depreciation)`, `General`, `Overhead`  |

Report totals:

- `gross_profit` = Trading Income − Cost of Sales
- `net_profit` = gross profit + Other Income − Operating Expenses

Every heading and subheading appears whether or not anything landed in it, so
the client renders the same skeleton each time. An empty one totals zero and
carries no accounts. Headings come in the order of the table above and a
heading's subheadings in the order listed there, whatever order the movements
arrived in; the client never sorts.

### Account totals

An account's total is its net movement over the period, signed and stated in
the direction the account normally runs. Which side that is belongs to the
class:

| Class     | Normally runs on |
| --------- | ---------------- |
| Asset     | Debit            |
| Equity    | Credit           |
| Expense   | Debit            |
| Liability | Credit           |
| Revenue   | Credit           |

So a revenue account totals credits − debits and an expense account debits −
credits, and a revenue account that was net refunded, or an expense account net
credited, totals negative. Headings and report totals sum the same way and may
also be negative.

Only revenue and expense reach this report, but the rule is stated once for
every class rather than twice for these two: the balance sheet nets the other
three the same way.

An account with line items in the period appears even when they net to zero. An
account with no line items in the period does not appear.

### Domain

- `model/signed_amount.rs` — `SignedAmount`, minor units, may be negative or
  zero. `Amount` cannot carry a direction, so a net movement needs a type of its
  own. Addition and subtraction saturate rather than wrap.
- `model/report_period.rs` — `ReportPeriod`, two `JournalDate`s, inclusive at
  both ends. Rejects a period whose end precedes its start.
- `model/account.rs` — `AccountClass` gains the side it normally runs on, one
  `EntryType` per class, per the table above.
- `model/account_movement.rs` — `AccountMovement`, an `Account` with the credits
  and debits posted to it over the period, and the net of the two taken in the
  direction its class normally runs. An `Amount` is positive by construction and
  a side with no postings totals zero, so the two sides are `SignedAmount`s too.
- `model/profit_and_loss.rs` — `ProfitAndLoss`, the assembled report: period,
  headings, and totals. Built by
  `ProfitAndLoss::for_period(period, movements)`, which classifies, groups,
  sorts each subheading by account code, and totals. This is pure, so it is
  where the report's logic lives and where it is unit tested.
- `port/journal_entry_repository.rs` — a new method on the existing port:

  ```rust
  fn net_movements<'a>(
      &'a self,
      period: &'a ReportPeriod,
  ) -> Pin<Box<dyn Future<Output = Result<Vec<AccountMovement>, NetMovementsError>> + Send + 'a>>;
  ```

  It reports every account with activity, not just the ones this report keeps,
  and one movement per account however many entries touched it. It totals each
  side and applies no direction of its own, so the sign stays with the class —
  the port is a ledger query rather than a report query, and the balance sheet
  reuses it. `ProfitAndLoss::for_period` discards the rest. `NetMovementsError`
  carries one variant, `Internal`.
- `use_case/generate_profit_and_loss.rs` — `GenerateProfitAndLossQuery` holds
  the period; the use case fetches the movements and builds the report. It holds
  no logic of its own beyond that. `GenerateProfitAndLossUseCaseError` carries
  `Repository`, which `NetMovementsError::Internal` becomes.

### Postgres adapter

- `net_movements` is one query: `journal_line_items` joined to `accounts`,
  filtered to `journal_entries.date` between the period's ends, grouped by
  account, summing the credits and the debits into a column each. It signs
  nothing; the class does that in the domain.
- `dto/account_movement.rs` — `AccountMovementPgRecord`, one grouped row: the
  account's own columns and both sums, converting into an `AccountMovement`.
- Postgres sums a `BIGINT` into a `NUMERIC`, which the `sqlx` features this
  crate enables cannot decode, so each sum comes back cast and defaulted —
  `COALESCE(SUM(...), 0)::BIGINT AS "credits!"` — to land in the `i64` the
  domain counts in.
- `NetMovementsError::Internal` covers both an `sqlx::Error` and a stored row
  that will not map back into a domain type, as the other repositories do.
  Nothing else escapes.
- New migration adding `CREATE INDEX journal_entries_date_idx ON
  journal_entries (date)`. It earns its keep on a period narrow against the
  ledger; a report covering most of the ledger scans whatever we do.

### HTTP adapter

`GET /reports/profit-and-loss?from=2026-01-01&to=2026-03-31`

Both parameters are required. `from` and `to` are the wire's names for the
period's ends; both are inclusive.

`200 OK`:

```json
{
  "period": { "from": "2026-01-01", "to": "2026-03-31" },
  "headings": [
    {
      "name": "Trading Income",
      "subheadings": [
        {
          "name": "Sales",
          "accounts": [
            { "code": "200", "description": "Sales revenue", "total": 150000 }
          ],
          "total": 150000
        }
      ],
      "total": 150000
    }
  ],
  "totals": { "gross_profit": 90000, "net_profit": 42000 }
}
```

- `handler/generate_profit_and_loss.rs` holds the handler and its own error
  enum, `GenerateProfitAndLossApiError`, as every endpoint does.
- `dto/profit_and_loss.rs` holds the response types, converting with
  `From<&ProfitAndLoss>`. An account that carries no description sends `null`,
  as an account does everywhere else on the wire.
- `dto/generate_profit_and_loss_request.rs` holds the query string and becomes
  `GenerateProfitAndLossQuery` through `TryFrom`. Both parameters arrive as
  strings and the domain parses them, so only an absent one is axum's to reject.
- `router.rs` gains the route, an `AppState` field, and a `FromRef`.

Statuses:

| Outcome                                | Status |
| -------------------------------------- | ------ |
| Report generated                       | 200    |
| `from` or `to` missing                 | 400 (axum's own rejection) |
| `from` or `to` not a date              | 422    |
| `to` before `from`                     | 422    |
| Repository failure                     | 500, cause hidden |

## Verification

### Happy paths

Domain unit tests, `ProfitAndLoss::for_period`:

- Groups accounts under the heading and subheading their class maps to.
- Totals a revenue account as credits less debits, an expense account as debits
  less credits.
- Totals each subheading, each heading, then gross profit and net profit.
- Totals a subheading, a heading, gross profit and net profit as negatives when
  what landed under them was net the other way.
- Sorts the accounts of a subheading by code.
- Returns the headings and subheadings in their fixed order, whatever order the
  movements arrive in.
- Returns every heading and subheading with a zero total and no accounts when
  nothing moved.
- Keeps an account whose movements net to zero.
- Excludes asset, liability, and equity movements.

`AccountMovement`: nets a credit-normal account as credits less debits and a
debit-normal account as debits less credits, for every class; nets a side with
no postings as zero.
`SignedAmount`: sums positives and negatives, holds a negative, holds zero,
saturates rather than wrapping at the ends of its range.
`ReportPeriod`: accepts a range, accepts a single day.

Use case integration test, `apps/api/tests/generate_profit_and_loss_use_case.rs`,
over a real Postgres:

- A period with sales and costs returns the report a caller expects — headings,
  account totals, gross profit, net profit.
- An entry dated on the first day, and one on the last, are both included.
- An entry outside the period is excluded.
- An account with no activity is absent.
- A period that also moved asset and liability accounts returns a report
  without them.
- A period with no entries returns the empty skeleton with zero totals.

Repository integration test, `apps/api/tests/postgres_journal_entry_repository.rs`:

- `net_movements` totals the credits and the debits of many entries against one
  account, and reports the account once.
- It bounds by date inclusively at both ends.
- It reports accounts of every class, asset, liability, and equity among them,
  the port being a ledger query.
- It reports an account whose credits and debits cancel.
- It returns an empty result for a period with no line items.

HTTP unit tests, beside the code:

- The response DTO maps a report — nesting, names, and totals — from a
  `ProfitAndLoss`.
- The response DTO maps a report nothing landed in: every heading and
  subheading, zero totals, no accounts.
- The response DTO sends `null` for an account with no description.
- The request DTO converts a well-formed query string into the domain query,
  `from` and `to` landing on the period's start and end.

### Unhappy paths

- `to` before `from` → `ReportPeriod` rejects it; the request DTO surfaces it as
  422 with the domain's message.
- `from=not-a-date` → 422.
- `from` absent → axum's rejection status, unremapped.
- Repository failure → 500 with `{"error": "internal server error"}` and no
  cause. Asserted by driving the endpoint's error enum through `IntoResponse`.
- Every status in the table above is asserted directly against the error enum,
  as the HTTP adapter gets no integration tests.
