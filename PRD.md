# PRD - Agent-Native Swarm Tool for Parallel Coding

## 1. Thong tin tai lieu

- Ten tam thoi: `Patchlane`
- Loai tai lieu: Product Requirements Document
- Trang thai: Draft v3
- Ngay cap nhat: 2026-03-09
- Muc tieu cua lan rewrite nay:
  - dua product thesis ve dung wedge ma user dang can;
  - cat bot framing "reliability layer" qua rong o surface chinh;
  - bien he thong thanh 1 cong cu swarm agent native trong coding-agent instance;
  - giu lai cac nang luc backend co gia tri thuc: planner, worktree isolation, merge control, recovery.

## 2. Tom tat dieu hanh

San pham nay khong nen duoc dinh vi nhu mot orchestration platform dashboard-first.

San pham nay nen duoc dinh vi nhu:

> Mot swarm tool native trong Coding Agent, cho phep user giao 1 objective va de he thong tu chay `spec -> shard -> assign -> worktree -> merge`, voi trang thai hien ngay trong chinh agent instance dang mo.

Gia tri cot loi:

- 1 lenh vao, nhieu assignment ben duoi;
- nhanh hon cach dieu phoi thu cong;
- re hon coordination bang tay;
- giam conflict nho shard + worktree + merge guard;
- operator van nam quyen dieu khien khi can.

Khac biet chinh:

- agent-native first, dashboard second;
- command-first, UI-second;
- speed va cost la uu tien cao nhat;
- planner va merge control phuc vu execution, khong phai de mo rong surface;
- web UI chi la tam nhin tong quan, khong phai noi user bat dau.

## 3. Van de can giai

Nguoi dung da quen voi Codex hoac Claude Code muon:

1. mo 1 coding-agent instance;
2. giao 1 objective bang ngon ngu tu nhien;
3. de he thong tu tach task, spawn agent, tao worktree, va hop code lai;
4. nhin thay trang thai ngay trong terminal/chat instance hien tai;
5. chi mo dashboard khi can xem tong quan hoac xem run cu.

Pain points hien tai:

1. Phai tu chia task va tu nho file overlap.
2. Phai copy context qua nhieu agent thu cong.
3. Khong ro agent nao dang lam gi, da block o dau, merge vi sao fail.
4. Qua nhieu thao tac phu quanh viec thuc thi.
5. Tool nhanh o demo nhung mat toc do va cost khi gap repo that.

## 4. Product thesis

Thesis moi:

> Teams va solo power users khong can mot "platform" to hon. Ho can mot command-native swarm tool ben trong coding-agent de bien 1 objective thanh mot run co spec, shard, assignment, worktree, va merge control, voi chi phi coordination toi thieu.

Khong dung positioning:

- AI swarm autopilot
- full engineering platform
- audit/compliance suite
- dashboard-first command center

Dung positioning:

- swarm tool for Codex / Claude Code
- agent-native parallel coding
- command-first task sharding and worktree orchestration

## 5. Muc tieu va phi muc tieu

### 5.1 Muc tieu san pham

1. User giao 1 objective tu chinh coding-agent instance.
2. He thong tu materialize objective thanh spec ngan du de shard.
3. He thong tao assignments an toan de chay song song.
4. Moi assignment duoc dispatch vao agent runtime/worktree phu hop.
5. User thay duoc run status, blocker, merge state ngay trong agent instance.
6. User co the intervene bang command ngan.
7. Toan bo flow nhanh, ro, va re hon cach lam thu cong.

### 5.2 Muc tieu ky thuat

1. Single entrypoint command cho run moi.
2. Execution placement phai thong minh: biet khi nao can worktree, khi nao co the xu ly ngay tren repo chinh.
3. Merge guard va recovery/resume phai co cho run dang do.
4. Planner phai giai thich duoc vi sao shard nhu vay.
5. Runtime support phai dung duoc tren Codex va Claude Code CLI.

### 5.3 Muc tieu UX

1. Primary UX trong coding-agent instance hien tai.
2. Moi update co dang ngan, scan nhanh, co hanh dong tiep theo ro.
3. Dashboard CLI hoac web chi doc projections on dinh, khong thay the luong chinh.

### 5.4 Phi muc tieu

1. Khong xay 10+ role phuc tap trong v1.
2. Khong xay team platform, auth, RBAC, remote control plane trong v1.
3. Khong day policy explorer, lineage explorer, compliance surface len luong chinh.
4. Khong toi uu cho multi-machine distributed execution trong v1.
5. Khong thay the Git hoac IDE.

## 6. Primary persona

### 6.1 Primary persona - solo/staff engineer da quen coding agents

Dac diem:

- dang su dung Codex hoac Claude Code hang ngay;
- muon giao 1 objective va de he thong tu orchestration;
- uu tien toc do va cost;
- khong muon hoc mot dashboard phuc tap truoc khi co gia tri;
- van muon thay va chenh duoc merge/blocker khi can.

Nhu cau:

- command surface ngan;
- ket qua scan duoc trong terminal;
- dashboard tong quan khi can;
- chi phi coordination thap;
- merge/worktree an toan du de tin dung.

### 6.2 Secondary persona - tech lead muon xem tong quan

Nhu cau:

- xem run nao dang chay;
- agent nao dang lam gi;
- task nao dang block;
- merge nao can approve;
- khong can can thiep vao tung chat transcript.

## 7. Jobs To Be Done

1. Khi toi dang o trong Codex hoac Claude Code, toi muon giao 1 objective va de tool tu tach viec cho cac agent khac.
2. Khi he thong dang chay, toi muon thay ngay trong phien hien tai no dang o buoc nao va task nao dang block.
3. Khi can, toi muon tam dung, retry, reassign, hoac approve merge bang command ngan.
4. Khi run xong, toi muon nhan ket qua hop nhat va biet da merge gi, bo gi, can quyet dinh gi.
5. Khi can nhin rong hon, toi muon mo CLI dashboard hoac web view de xem toan canh.

## 8. Nguyen tac san pham

1. Agent-native first.
2. Command-first, dashboard-second.
3. 1 objective vao, nhieu assignments ra.
4. Prevention hon merge rescue.
5. Worktree la phuong tien, khong phai muc tieu; chi tao khi no tang an toan hon chi phi no gay ra.
6. Chi materialize nhung artifact can thiet cho execution va recovery.
7. Moi trang thai hien ra phai tra loi duoc: dang lam gi, bi chan boi cai gi, buoc tiep theo la gi.
8. Moi do phuc tap them vao phai chung minh tang toc do hoac giam cost.
9. Mac dinh la `flat swarm`; cac mode khac chi them khi co evidence.

## 9. Core loop

Core loop cua san pham:

1. User nhap objective trong coding-agent instance.
2. He thong tao `spec draft`.
3. He thong chay `check/clarify` neu objective con mo ho.
4. He thong shard thanh assignments.
5. He thong score overlap risk va quyet dinh shard nao chay song song.
6. He thong tao worktree cho moi assignment duoc dispatch.
7. Builder agents thuc thi.
8. Reviewer agent hoac merge checks danh gia.
9. Merge queue xu ly merge.
10. Ket qua va next actions duoc stream lai trong chinh agent instance.

## 9.1 Execution placement policy

Khong phai moi assignment deu phai tao worktree.

He thong phai co 3 placement modes:

- `main_repo`: thuc thi ngay trong repo chinh hien tai;
- `worktree`: tao workspace rieng cho assignment;
- `blocked`: khong dispatch vi rui ro qua cao hoac repo state khong an toan.

### Nguyen tac

1. Neu co hon 1 writable shard chay song song, `worktree` la mac dinh.
2. Neu assignment co overlap risk cao, `worktree` hoac `blocked`, khong duoc `main_repo`.
3. Neu objective rat nho, single-shard, khong parallel, va working tree dang sach, system co the chon `main_repo` de tiet kiem chi phi setup.
4. Neu repo dang dirty, co untracked files nhay cam, hoac co state khong on dinh, system khong duoc tu y chay tren `main_repo`.
5. Neu mode la `fast`, system co the uu tien `main_repo` cho low-risk single-assignment runs.
6. Neu mode la `safe`, system phai bias manh ve `worktree`.
7. Moi quyet dinh placement phai duoc giai thich ro cho operator.

### Tin hieu ra quyet dinh

He thong nen can nhac toi thieu:

- so shard writable;
- co parallel hay khong;
- overlap risk;
- expected files co dung hotspot hay khong;
- repo clean/dirty status;
- co merge queue dang mo hay khong;
- execution mode `fast|balanced|safe`;
- cost cua viec tao worktree so voi cost coordination du kien.

### Operator-facing rule

User khong nhat thiet phai chi dinh worktree.

User giao objective, he thong tra loi:

- shard nao se chay o `main_repo`;
- shard nao se chay trong `worktree`;
- shard nao bi block cho toi khi user xac nhan.

### Guardrails

`main_repo` chi duoc phep khi tat ca dieu kien sau dung:

- run co 1 writable shard dang active tai mot thoi diem;
- khong co overlap voi assignment khac dang chay;
- working tree sach hoac nam trong nguong ma system da hieu ro;
- objective nam trong low-risk envelope;
- merge strategy don gian, khong can rebase/resolve phuc tap;
- runtime khong bi degrade o muc lam provenance mat tin cay.

Neu mot trong cac dieu kien tren khong dung, system phai chuyen sang `worktree` hoac `blocked`.

## 10. Command surface chi tiet

### 10.1 Design principle

User khong nen can biet noi bo co bao nhieu script, policy object, hay runtime adapter.

Surface chinh chi nen gom 3 nhom lenh:

- run
- observe
- intervene

### 10.2 Lenh chinh

#### `swarm run`

Muc dich:
- tao run moi tu objective;
- materialize spec;
- shard;
- assign runtime;
- tao worktree;
- dispatch execution;
- theo doi cho den khi can intervention hoac complete.

Vi du:

```bash
swarm run "implement a simple swarm tool that breaks task and assigns agents"
```

Flags v1:

- `--repo <path>`: chay voi repo cu the
- `--runtime <codex|claude|auto>`: chon runtime uu tien
- `--max-agents <n>`: gioi han builder song song
- `--mode <fast|balanced|safe>`: trade-off speed/cost/risk
- `--placement <auto|main_repo|worktree>`: cho phep operator ep placement strategy neu can
- `--auto-merge <off|safe>`: co tu merge cac shard clean hay khong
- `--budget <value>`: tran chi phi/step budget cho run
- `--non-interactive`: fail thay vi hoi them clarification

Output trong agent instance:

- spec summary
- shard count
- placement plan cho tung shard
- shards dang chay / dang doi / bi block
- worktree duoc tao cho shard nao
- merge queue status
- next action neu can user can thiep

#### `swarm status`

Muc dich:
- xem trang thai 1 run dang chay hoac run gan nhat.

Vi du:

```bash
swarm status
swarm status <run-id>
```

Noi dung:

- run state
- tung shard va runtime dang gan
- placement cua tung shard
- blocker
- merge status
- chi phi tom tat
- latest event

#### `swarm watch`

Muc dich:
- stream update lien tuc cho 1 run.

Vi du:

```bash
swarm watch
swarm watch <run-id>
```

Noi dung:

- event stream dang ngan
- state transitions
- warnings
- merge decisions can operator

### 10.3 Lenh intervention

#### `swarm pause <run-id|shard-id>`
#### `swarm resume <run-id|shard-id>`
#### `swarm retry <shard-id>`
#### `swarm reassign <shard-id> --runtime <codex|claude>`
#### `swarm merge approve <merge-unit-id>`
#### `swarm merge reject <merge-unit-id>`
#### `swarm stop <run-id>`

V1 yeu cau:

- command ngan;
- idempotent;
- response ro `queued|acknowledged|applied|failed`;
- khong bat user mo dashboard moi can thiep duoc.

### 10.4 Lenh overview phu

#### `swarm board`

Muc dich:
- mo CLI dashboard nhe de xem nhieu run.

Noi dung:

- run list
- active shards
- blocked shards
- merge queue
- top warnings

#### `swarm web`

Muc dich:
- mo web overview khi can view rong hon.

Pham vi:

- read-mostly overview;
- merge queue;
- shard detail;
- khong la entry surface mac dinh.

## 11. Surfaces va do uu tien

### 11.1 Surface uu tien 1 - coding-agent instance

Day la noi user bat dau va noi user theo doi run.

Bat buoc co:

- `swarm run`
- progress summary
- blocker summary
- merge prompt
- intervention command suggestions

### 11.2 Surface uu tien 2 - terminal overview

Day la noi user muon xem tong quan nhanh ma khong can web browser.

Bat buoc co:

- bang run/shard/merge;
- refresh nhanh;
- scan trong 1 man hinh.

### 11.3 Surface uu tien 3 - web overview

Chi mo khi can:

- xem tong quan nhieu run;
- mo merge detail;
- xem lich su;
- debug thong tin dai.

Web khong duoc lai tro thanh noi user phai vao de bat dau 1 run.

## 12. Scope v1

V1 tap trung vao:

- 1 command entrypoint `swarm run`
- spec draft + shard draft + overlap scoring
- execution placement intelligence `main_repo|worktree|blocked`
- builder/reviewer capability co ban
- worktree lifecycle khi placement chon `worktree`
- main-repo guarded execution khi placement chon `main_repo`
- merge queue va merge decision
- resume/recovery
- in-agent progress updates
- terminal overview
- web overview toi thieu

V1 chua bao gom:

- auth/RBAC
- remote multi-operator
- advanced analytics
- policy explorer rieng
- lineage explorer day du
- compliance/audit suite

## 13. Kien truc v1

### 13.1 Thanh phan giu lai

- planner
- orchestrator
- worktree isolation
- repo-state inspection va placement decision
- merge engine
- recovery
- runtime adapters

### 13.2 Thanh phan can ha uu tien o surface

- policy explorer rich UI
- cost analytics sau
- audit export day du
- lineage explorer
- clarification UI phuc tap

### 13.3 Source of truth

- run
- shard
- assignment
- assignment attempt
- merge unit
- intervention

Tat ca surface chi doc tu projections on dinh cua cac entity nay.

## 14. Roadmap moi

### Phase 0 - Realign product and shell integration

Muc tieu:
- doi thesis sang agent-native swarm tool;
- chot command surface;
- chot shell integration cho Codex/Claude Code.

### Phase 1 - Single-command swarm execution

Muc tieu:
- `swarm run` tao spec, shard, chon placement, assign, dispatch;
- stream status ve ngay trong agent instance;
- co intervention co ban.

### Phase 2 - Planner quality and merge confidence

Muc tieu:
- shard tot hon;
- giam overlap;
- merge checks ro hon;
- cost/risk mode ro hon.

### Phase 3 - Runtime parity and cost control

Muc tieu:
- Codex va Claude Code chay cung command surface;
- parity du de tin dung;
- quan ly degradation va budget.

### Phase 4 - Overview surfaces

Muc tieu:
- terminal overview hoan chinh;
- web overview read-mostly;
- merge queue va shard detail tot hon.

### Phase 5 - Team product neu can

Muc tieu:
- chi mo khi wedge v1 da duoc xac nhan co gia tri.

## 15. Acceptance criteria

San pham duoc xem la dung huong khi:

1. User co the mo Codex hoac Claude Code va go 1 objective.
2. He thong tu tao spec/shards ma khong bat user phai hoc workflow rieng.
3. User thay duoc trang thai thuc thi ngay trong phien agent.
4. User thay duoc vi sao shard nao dung `main_repo`, shard nao dung `worktree`.
5. User co the pause/retry/reassign/merge bang command ngan.
6. User co the mo CLI board hoac web overview de xem tong quan.
7. Tong overhead coordination nho hon cach lam thu cong.

## 16. Chi so thanh cong

Chi so chinh:

- time-to-first-shard
- time-to-first-merge
- total run duration
- placement accuracy: ti le run low-risk duoc xu ly o `main_repo` ma khong tang loi
- operator interventions per run
- blocked shard rate
- merge conflict rate
- estimated coordination cost

Chi so loai bo:

- so trang dashboard
- so loai report
- so surface policy/audit rieng

## 17. Quy tac scope

1. Bat ky task nao khong giup `swarm run/status/watch/intervene/board/web` manh hon thi mac dinh la follow-up.
2. Bat ky surface nao yeu cau user roi coding-agent instance de bat dau run thi la scope sai.
3. Bat ky do phuc tap nao khong chung minh giup nhanh hon hoac re hon thi phai cat.
4. Khong duoc hardcode `luon tao worktree`; placement phai la quyet dinh thong minh, co giai thich va co guardrails.
