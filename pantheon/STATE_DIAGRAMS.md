# Pantheon State Diagrams
## Behavioral Models of All Core Subsystems

**Document**: 8 of 20
**Version**: Genesis
**Status**: Architectural Specification

---

## 1. System-Level State Machine

```
Pantheon System Lifecycle:

                    ┌─────────────┐
                    │  BOOTING    │
                    │  (Kernel    │
                    │   init)     │
                    └──────┬──────┘
                           │ kernel_ready
                           ▼
                    ┌─────────────┐
                    │  INITIALIZING│
                    │  (Services  │
                    │   start)    │
                    └──────┬──────┘
                           │ services_ready
                           ▼
                    ┌─────────────┐
         ┌─────────│   RUNNING   │──────────┐
         │         │  (Normal    │          │
         │         │   operation)│          │
         │         └─────────────┘          │
         │                                  │
         ▼                                  ▼
  ┌─────────────┐                   ┌─────────────┐
  │ MAINTENANCE │                   │  EMERGENCY  │
  │ (Update,    │                   │ (Incident   │
  │  migration) │                   │  response)  │
  └──────┬──────┘                   └──────┬──────┘
         │                                  │
         ▼                                  ▼
  ┌─────────────┐                   ┌─────────────┐
  │  RESUMING   │                   │  RECOVERING │
  └──────┬──────┘                   └──────┬──────┘
         │                                  │
         └──────────┬──────────────────────┘
                    ▼
             ┌─────────────┐
             │   RUNNING   │
             └─────────────┘
                    │
                    │ shutdown
                    ▼
             ┌─────────────┐
             │  SHUTDOWN   │
             └─────────────┘
```

---

## 2. Cell State Machine

```
Cell Lifecycle (Detailed):

              ┌─────────────────────────────────────┐
              │                                     │
              ▼                                     │
        ┌──────────┐                         ┌──────────┐
    ┌──→│ CREATED  │────────────────────────→│ RUNNING  │──┐
    │   └──────────┘   start                  └────┬─────┘  │
    │         │                                    │        │
    │         │ create                              │        │
    │         ▼                                    │        │
    │   ┌──────────┐    suspend              ┌─────▼──────┐ │
    │   │  NULL    │──────────────────────→│ SUSPENDED  │ │
    │   └──────────┘                        └─────┬──────┘ │
    │                                             │        │
    │         ┌───────────────────────────────────┘        │
    │         │   resume                                   │
    │         ▼                                            │
    │   ┌──────────┐               block            ┌──────▼───┐
    │   │ RUNNING  │──────────────────────────────→│ BLOCKED  │
    │   └────┬─────┘                                └─────┬────┘
    │         │                                            │
    │         │ terminate                    unblock       │
    │         ▼                                            │
    │   ┌──────────┐                                       │
    │   │TERMINATED│◄──────────────────────────────────────┘
    │   └────┬─────┘
    │         │
    │         │ retire
    │         ▼
    │   ┌──────────┐
    │   │ RETIRED  │
    │   └────┬─────┘
    │         │
    │         │ destroy
    │         ▼
    │   ┌──────────┐
    └───│ DESTROYED│
        └──────────┘

Transitions:
  NULL        → CREATED    : on create_cell()
  CREATED     → RUNNING    : on start_cell()
  CREATED     → TERMINATED : on terminate_cell() (never started)
  RUNNING     → SUSPENDED  : on suspend_cell() or preemption
  SUSPENDED   → RUNNING    : on resume_cell()
  RUNNING     → BLOCKED    : on block() (waiting for event)
  BLOCKED     → RUNNING    : on unblock() (event received)
  RUNNING     → TERMINATED : on terminate_cell() or fail()
  BLOCKED     → TERMINATED : on terminate_cell()
  SUSPENDED   → TERMINATED : on terminate_cell()
  TERMINATED  → RETIRED    : on retire_cell()
  RETIRED     → DESTROYED  : on destroy_cell() (GC)
  DESTROYED   → CREATED    : on recreate_cell() (if needed)
```

---

## 3. Event Bus Partition State Machine

```
Partition Lifecycle:

              ┌────────────┐
              │ UNASSIGNED │
              └──────┬─────┘
                     │ assign
                     ▼
              ┌────────────┐
         ┌───→│  FOLLOWER  │
         │    └─────┬──────┘
         │          │ election
         │          ▼
         │    ┌────────────┐
         │    │  CANDIDATE │
         │    └─────┬──────┘
         │          │ elected
         │          ▼
         │    ┌────────────┐
         │ ┌─→│  LEADER    │
         │ │  └─────┬──────┘
         │ │        │ lost leadership
         │ │        ▼
         │ │  ┌────────────┐
         │ └──│  FOLLOWER  │
         │    └────────────┘
         │
         │         │ partition merge
         │         ▼
         │    ┌────────────┐
         └────│  MERGING   │
              └─────┬──────┘
                    │ merged
                    ▼
              ┌────────────┐
              │ UNASSIGNED │
              └────────────┘
```

---

## 4. Task State Machine

```
Task Lifecycle:

        ┌──────────┐
        │ CREATED  │
        └────┬─────┘
             │ submit
             ▼
        ┌──────────┐
        │  QUEUED  │◄─────────────┐
        └────┬─────┘               │
             │ dequeue             │ retry
             ▼                     │
        ┌──────────┐               │
        │ ASSIGNED │───────────────┤
        └────┬─────┘  fail        │
             │ start              │
             ▼                    │
        ┌──────────┐              │
        │ RUNNING  │──────────────┤
        └────┬─────┘  fail        │
             │                    │
        ┌────┴────┐              │
        ▼         ▼              │
   ┌────────┐ ┌────────┐        │
   │SUCCEEDED│ │ FAILED │────────┘
   └─────────┘ └────────┘   retry
         │           │
         │           │ retry exhausted
         ▼           ▼
   ┌─────────┐ ┌──────────┐
   │COMPLETED│ │ ABANDONED│
   └─────────┘ └──────────┘
```

---

## 5. Proposal State Machine (Governance)

```
Proposal Lifecycle:

        ┌──────────┐
        │  DRAFT   │◄──────────┐
        └────┬─────┘            │
             │ submit           │ revise
             ▼                  │
        ┌──────────┐            │
        │ SUBMITTED│────────────┘
        └────┬─────┘
             │ review
             ▼
        ┌──────────┐
        │ UNDER    │
        │ REVIEW   │
        └────┬─────┘
        ┌────┴────┐
        ▼         ▼
   ┌────────┐ ┌────────┐
   │APPROVED│ │REJECTED│
   │ (to    │ └────────┘
   │ debate)│
   └────┬───┘
        │ debate
        ▼
   ┌──────────┐
   │  DEBATE  │
   └────┬─────┘
        │ vote
        ▼
   ┌──────────┐
   │  VOTING  │
   └────┬─────┘
   ┌────┴───────┐
   ▼            ▼
┌────────┐ ┌────────┐
│PASSED  │ │ FAILED │
└───┬────┘ └────────┘
    │ enact
    ▼
┌──────────┐
│ ENACTED  │
└────┬─────┘
 │ review
 │ (optional)
 ▼
┌──────────┐
│  REVIEW  │
└────┬─────┘
     │ outcome
     ├──────────────┐
     ▼              ▼
┌──────────┐  ┌──────────┐
│SUSTAINED │  │ REPEALED │
└──────────┘  └──────────┘
```

---

## 6. Agent State Machine

```
Agent Lifecycle (Detailed):

        ┌──────────┐
        │   SEED   │
        └────┬─────┘
             │ curriculum
             ▼
        ┌──────────┐
        │ TRAINING │
        └────┬─────┘
  ┌──────────┼──────────┐
  ▼          ▼          ▼
┌────────┐┌────────┐┌────────┐
│CERTIFIED││ FAILED ││EXPERI- │
│         ││(retry) ││MENTAL  │
└───┬────┘└────────┘└───┬────┘
    │                     │ promote
    ▼                     ▼
┌──────────┐        ┌──────────┐
│  ACTIVE  │        │  ACTIVE  │
└────┬─────┘        │ (limited)│
     │              └──────────┘
     │
     ├──────────┬──────────┬──────────┐
     ▼          ▼          ▼          ▼
┌────────┐┌────────┐┌────────┐┌─────────┐
│REVIEW  ││ SPLIT  ││ MERGE  ││ UPGRADE │
└───┬────┘└───┬────┘└───┬────┘└────┬────┘
    │         │         │          │
    │         ▼         ▼          ▼
    │    ┌────────┐┌────────┐┌──────────┐
    │    │ACTIVE(A)││ ACTIVE ││ ACTIVE   │
    │    │ACTIVE(B)││(merged)││(upgraded)│
    │    └────────┘└────────┘└──────────┘
    │
    │ decision
    ├──────────────┐
    ▼              ▼
┌────────┐  ┌──────────┐
│PROMOTED│  │ RETIRED  │
│(wider  │  │(archived)│
│ scope) │  └──────────┘
└────────┘
```

---

## 7. Plugin Lifecycle State Machine

```
Plugin Lifecycle:

        ┌─────────────┐
        │ DISCOVERED  │
        └──────┬──────┘
               │ verify
               ▼
        ┌─────────────┐
        │  VERIFIED   │
        └──────┬──────┘
   ┌───────────┼───────────┐
   ▼           ▼           ▼
┌────────┐┌────────┐┌──────────┐
│INSTALLED││REJECTED││DEPRECATED│
└────┬───┘└────────┘└──────────┘
     │ enable
     ▼
┌──────────┐
│  ENABLED │
└────┬─────┘
     │ activate
     ▼
┌──────────┐
│ RUNNING  │
└────┬─────┘
     │
     ├──────────────┬──────────────┐
     ▼              ▼              ▼
┌────────┐  ┌────────────┐  ┌──────────┐
│DISABLED│  │  FAILED    │  │UPDATING  │
└────┬───┘  └──────┬─────┘  └────┬─────┘
     │             │             │
     │             │ rollback    │ updated
     │             ▼             ▼
     │        ┌──────────┐  ┌──────────┐
     │        │ROLLED_BACK│ │ UPDATED  │
     │        └──────────┘  └────┬─────┘
     │                           │ restart
     ▼                           ▼
┌──────────┐              ┌──────────┐
│  ENABLED │              │ RUNNING  │
└──────────┘              └──────────┘
```

---

## 8. Event Bus Consumer State Machine

```
Consumer Lifecycle:

        ┌──────────┐
        │ CREATED  │
        └────┬─────┘
             │ connect
             ▼
        ┌──────────┐
        │CONNECTING│
        └────┬─────┘
             │ connected
             ▼
        ┌──────────┐
        │ CONNECTED│
        └────┬─────┘
             │ subscribe
             ▼
        ┌──────────┐
        │STREAMING │◄────────────────┐
        └────┬─────┘                  │
             │                       │
        ┌────┴────┐                  │
        ▼         ▼                  │
   ┌────────┐┌────────┐             │
   │ACTIVE  ││ PAUSED │─────────────┘
   │(consumer││(back-  │  resume
   │leader) ││pressure)│
   └────┬───┘└────────┘
        │
        │
   ┌────┴────┐
   ▼         ▼
┌────────┐┌────────┐
│REBALANCE││ CLOSED │
│(group   │└────────┘
│change)  │
└────┬───┘
     │ complete
     ▼
┌────────┐
│ ACTIVE │
└────────┘
```
