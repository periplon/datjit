# DDL — Data Domain Language v0.1

A compact language for defining data domains with enough semantic and statistical hints to enable high-quality synthetic data generation and automatic tool surface inference.

---

## 1. Document Structure

```yaml
domain: <name>                    # required — domain identifier
version: <semver>                 # optional — schema version
seed: <int>                       # optional — deterministic generation
locale: <bcp47>                   # optional — default "en-US"
volume:                           # optional — generation scale
  <Entity>: <int|range>           # target row counts per entity

entities:
  <EntityName>:                   # PascalCase
    <field>: <type> <decorators>  # fields

enums:                            # optional — shared enum definitions
  <EnumName>: [val, val, ...]

types:                            # optional — reusable compound types
  <TypeName>:
    <field>: <type> <decorators>

rules:                            # optional — cross-entity constraints
  - <rule_expression>

tools:                            # optional — override auto-inferred tools
  <EntityName>:
    <override_block>
```

---

## 2. Type System

### 2.1 Primitive Types

| Type        | Description                    | Default generator         |
|-------------|--------------------------------|---------------------------|
| `string`    | UTF-8 text                     | random alphanum           |
| `int`       | 64-bit signed integer          | uniform random            |
| `float`     | 64-bit IEEE 754                | uniform random            |
| `bool`      | true / false                   | 50/50                     |
| `datetime`  | ISO 8601 timestamp             | uniform in range          |
| `date`      | ISO 8601 date only             | uniform in range          |
| `time`      | ISO 8601 time only             | uniform in range          |
| `duration`  | ISO 8601 duration              | uniform in range          |
| `uuid`      | UUID v4                        | random                    |
| `bytes`     | base64-encoded binary          | random                    |
| `null`      | explicit null                  | always null               |
| `any`       | untyped / opaque JSON          | —                         |

### 2.2 Parameterized Types

```
int(32)                   # bit-width hint
float(32)                 # bit-width hint
string(maxlen)            # max character length
bytes(maxlen)             # max byte length
decimal(precision, scale) # fixed-point: decimal(10,2)
```

### 2.3 Compound Types

```
[T]                       # list of T
{K: V}                    # map with key type K, value type V
(T1, T2, ...)             # tuple
T?                        # nullable (equivalent to T | null)
T1 | T2                   # union type
```

### 2.4 Enum Types

Inline:
```yaml
status: enum(active, inactive, suspended)
```

Named (reusable):
```yaml
enums:
  Color: [red, green, blue, yellow]

entities:
  Widget:
    color: Color
```

### 2.5 Reference Types

```yaml
customer: ->Customer              # required foreign reference
customer: ->Customer?             # optional foreign reference
tags: ->[Tag]                     # list of references
parent: ->self                    # self-referential
```

### 2.6 Semantic Types

Semantic types replace bare primitives with domain-meaningful tags that guide data generation. They use dot-notation namespaces.

#### Person
| Tag                  | Output example                  |
|----------------------|---------------------------------|
| `person.full`        | "Maria Santos"                  |
| `person.first`       | "Maria"                         |
| `person.last`        | "Santos"                        |
| `person.prefix`      | "Dr."                           |
| `person.suffix`      | "Jr."                           |
| `person.username`    | "msantos42"                     |
| `person.bio`         | paragraph-length bio text       |
| `person.avatar`      | URL to generated avatar         |
| `person.gender`      | "female" / "male" / "nonbinary" |
| `person.dob`         | "1987-03-14"                    |
| `person.age`         | 36                              |
| `person.ssn`         | "XXX-XX-XXXX" (locale-aware)    |

#### Contact
| Tag                  | Output example                  |
|----------------------|---------------------------------|
| `email`              | "msantos@example.com"           |
| `phone`              | "+1-555-867-5309"               |
| `phone.mobile`       | "+1-555-867-5309"               |
| `phone.landline`     | "+1-555-234-5678"               |
| `url`                | "https://example.com/page"      |
| `url.image`          | "https://picsum.photos/400/300" |
| `url.avatar`         | gravatar-style URL              |
| `ipv4`               | "192.168.1.42"                  |
| `ipv6`               | "2001:0db8:..."                 |
| `mac`                | "3D:F2:C9:A6:..."               |

#### Location
| Tag                  | Output example                  |
|----------------------|---------------------------------|
| `address.full`       | "742 Evergreen Terrace, ..."    |
| `address.street`     | "742 Evergreen Terrace"         |
| `address.city`       | "Springfield"                   |
| `address.state`      | "IL"                            |
| `address.zip`        | "62704"                         |
| `address.country`    | "US"                            |
| `geo.lat`            | 39.7817                         |
| `geo.lng`            | -89.6501                        |
| `geo.point`          | (39.7817, -89.6501)             |
| `timezone`           | "America/Chicago"               |

#### Finance
| Tag                  | Output example                  |
|----------------------|---------------------------------|
| `currency.usd`       | 49.99                           |
| `currency.eur`       | 42.50                           |
| `currency(CODE)`     | locale-formatted amount         |
| `credit_card`        | "4111-1111-1111-1111"           |
| `credit_card.type`   | "visa"                          |
| `iban`               | "DE89370400440532013000"        |
| `swift`              | "COBADEFFXXX"                   |

#### Text / Content
| Tag                  | Output example                  |
|----------------------|---------------------------------|
| `text.word`          | "ephemeral"                     |
| `text.sentence`      | one sentence                    |
| `text.paragraph`     | one paragraph                   |
| `text.paragraphs(n)` | n paragraphs                    |
| `text.slug`          | "my-great-post"                 |
| `text.markdown`      | formatted markdown content      |
| `text.html`          | formatted HTML content          |
| `text.lorem(n)`      | n words of lorem ipsum          |

#### Domain-Specific Content
| Tag                    | Output example                |
|------------------------|-------------------------------|
| `product.title`        | "Wireless Noise-Canceling..." |
| `product.description`  | product marketing copy        |
| `product.sku`          | "SKU-EL-4829"                 |
| `company.name`         | "Apex Dynamics Inc."          |
| `company.industry`     | "Manufacturing"               |
| `company.catch_phrase`  | "Innovate. Integrate. Excel." |
| `job.title`            | "Senior Software Engineer"    |
| `job.department`       | "Engineering"                 |
| `color.hex`            | "#3B82F6"                     |
| `color.rgb`            | "rgb(59, 130, 246)"           |
| `color.name`           | "cerulean"                    |
| `file.name`            | "report_q3.pdf"               |
| `file.extension`       | ".pdf"                        |
| `file.mime`            | "application/pdf"             |

#### Identifiers
| Tag                  | Output example                  |
|----------------------|---------------------------------|
| `sku`                | "SKU-AB-1234"                   |
| `slug`               | "my-item-name"                  |
| `code`               | "ABC123"                        |
| `hash.md5`           | 32 hex chars                    |
| `hash.sha256`        | 64 hex chars                    |

#### Custom Semantic Tags

Use the `@domain()` decorator to scope a semantic type:

```yaml
name: product.title @domain(electronics)    # "Bluetooth Speaker Pro X"
name: product.title @domain(food)           # "Organic Acai Bowl Mix"
```

### 2.7 Field Labels and Descriptions

Fields support an optional `label` (human-readable name) and `description` (documentation text). To use them, write the field as a mapping with a `type` key instead of the shorthand string:

```yaml
entities:
  Material:
    id: uuid @primary                              # shorthand — still works
    number:
      type: string @unique @pattern("MAT-{0000000}")
      label: "Material Number"
      description: "Unique identifier assigned by the ERP system"
    name:
      type: product.title
      label: "Material Name"
    inspection_required:
      type: enum(0, 1) @dist(80, 20)
      description: "Whether incoming inspection is mandatory"
```

Both `label` and `description` are optional and can be used independently. They appear in `inspect` output and serve as documentation — they do not affect generation.

---

## 3. Decorators

Decorators are annotations prefixed with `@` that modify field behavior for generation, validation, and tool inference. Multiple decorators are space-separated.

### 3.1 Identity & Uniqueness

| Decorator         | Meaning                                       |
|-------------------|-----------------------------------------------|
| `@auto`           | System-generated, excluded from create inputs  |
| `@unique`         | All generated values are unique                |
| `@primary`        | Primary key (implies @auto @unique)            |
| `@index`          | Field is indexed / filterable in queries       |
| `@immutable`      | Set once, excluded from update inputs          |

### 3.2 Value Constraints

| Decorator                 | Meaning                                    |
|---------------------------|--------------------------------------------|
| `@range(lo..hi)`          | Inclusive numeric or date range             |
| `@range(2020..now)`       | `now` = generation timestamp               |
| `@min(n)`                 | Minimum value                              |
| `@max(n)`                 | Maximum value                              |
| `@len(lo..hi)`            | String/list length range                   |
| `@pattern("regex")`       | Value must match regex                     |
| `@pattern("SKU-{AA}-{0000}")` | Template pattern (see §3.7)           |
| `@values(a,b,c)`          | Allowed values (non-enum shorthand)       |
| `@not_empty`              | String/list must have length ≥ 1           |
| `@optional`               | Field may be omitted (null)                |
| `@default(val)`           | Default value if not provided              |

### 3.3 Distribution Hints

Control statistical shape of generated data.

| Decorator                       | Meaning                                     |
|---------------------------------|---------------------------------------------|
| `@dist(p1, p2, ...)`           | Categorical probabilities for enum values (%) |
| `@dist(uniform)`               | Uniform random (default)                     |
| `@dist(normal, μ=X, σ=Y)`     | Normal / Gaussian                            |
| `@dist(lognormal)`             | Log-normal (right-skewed)                    |
| `@dist(lognormal, μ=X, σ=Y)`  | Parameterized log-normal                     |
| `@dist(exponential, λ=X)`     | Exponential decay                            |
| `@dist(geometric, p=X)`       | Geometric (discrete)                         |
| `@dist(zipf, s=X)`            | Zipf / power-law                             |
| `@dist(bimodal, peaks=X,Y)`   | Two-peak distribution                        |
| `@dist(weighted, {v1: w, v2: w})` | Weighted discrete values                 |
| `@null_rate(p)`                | Probability field is null (0.0–1.0)          |

**Enum distribution shorthand:**
```yaml
tier: enum(free, pro, enterprise) @dist(70, 25, 5)
# 70% free, 25% pro, 5% enterprise
```

**Numeric distributions:**
```yaml
price: currency.usd @range(1..5000) @dist(lognormal, μ=3.5, σ=1.2)
age: int @range(18..95) @dist(normal, μ=35, σ=12)
```

### 3.4 Relational Decorators

| Decorator                     | Meaning                                     |
|-------------------------------|---------------------------------------------|
| `@count(lo..hi)`              | Number of related items to generate         |
| `@count(n)`                   | Exact count                                 |
| `@from(field)`                | Value derived/correlated with another field  |
| `@after(ref.field)`           | Datetime must be after referenced field      |
| `@before(ref.field)`          | Datetime must be before referenced field     |
| `@within(duration, ref)`      | Datetime within duration of reference        |
| `@correlated(field, r=0.8)`  | Statistical correlation with another field   |

### 3.5 Derivation

Derived fields are computed, not stored. Excluded from create/update inputs.

```yaml
subtotal: currency.usd @derived(qty * product.price)
full_name: string @derived(concat(first_name, " ", last_name))
age: int @derived(years_since(dob))
item_count: int @derived(count(items))
total: currency.usd @derived(sum(items.subtotal))
is_premium: bool @derived(tier in [pro, enterprise])
```

**Derivation functions:**

| Function                 | Meaning                            |
|--------------------------|------------------------------------|
| `concat(a, sep, b, ...)` | String concatenation              |
| `sum(collection.field)`  | Sum over related collection        |
| `count(collection)`      | Count of related items             |
| `avg(collection.field)`  | Average over collection            |
| `min(collection.field)`  | Minimum in collection              |
| `max(collection.field)`  | Maximum in collection              |
| `years_since(date)`      | Integer years from date to now     |
| `days_between(a, b)`     | Integer days between two dates     |
| `if(cond, then, else)`   | Conditional                        |
| `round(expr, decimals)`  | Rounding                           |
| `lower(s)` / `upper(s)` | Case transform                     |
| `slug(s)`                | Slugify string                     |

Arithmetic operators: `+`, `-`, `*`, `/`, `%`
Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
Logic: `and`, `or`, `not`, `in`

### 3.6 Tool Behavior Decorators

| Decorator             | Meaning                                          |
|-----------------------|--------------------------------------------------|
| `@readonly`           | Entity-level: no create/update/delete tools       |
| `@no_delete`          | No delete tool generated                          |
| `@soft_delete`        | Delete sets a flag instead of removing             |
| `@sortable`           | Field appears in sort options                      |
| `@filterable`         | Field appears in filter options                    |
| `@searchable`         | Field included in full-text search                 |
| `@hidden`             | Field excluded from list/summary responses         |
| `@sensitive`          | Field masked in responses unless explicitly fetched|
| `@paginated(size)`    | Default page size for list tool                    |

### 3.7 Pattern Templates

Pattern templates use `{}` placeholders for structured identifiers:

| Placeholder | Meaning                    | Example   |
|-------------|----------------------------|-----------|
| `{A}`       | One uppercase letter       | "K"       |
| `{AA}`      | Two uppercase letters      | "KM"      |
| `{a}`       | One lowercase letter       | "k"       |
| `{0}`       | One digit                  | "7"       |
| `{0000}`    | N digits, zero-padded      | "0042"    |
| `{####}`    | N hex digits               | "3FA1"    |
| `{word}`    | One lowercase word         | "alpha"   |
| `{WORD}`    | One uppercase word         | "ALPHA"   |
| `{uuid}`    | UUID v4                    | "a3f1..." |
| `{seq}`     | Auto-incrementing integer  | "1"       |

```yaml
sku: string @pattern("SKU-{AA}-{0000}")     # "SKU-KM-0042"
code: string @pattern("{AAA}-{000}")          # "FGX-014"
slug: string @pattern("{word}-{word}-{0000}") # "blue-widget-0391"
```

---

## 4. Relationships

### 4.1 Reference Syntax

```yaml
# Belongs-to (foreign key on this entity)
author: ->User                    # required reference
reviewer: ->User?                 # optional reference
parent: ->self                    # self-referential tree

# Has-many (reverse side; foreign key on target)
posts: [Post]                     # one-to-many
posts: [Post] @count(0..20)       # with cardinality hint

# Many-to-many (implicit join entity)
tags: <->Tag                      # bidirectional many-to-many
tags: <->Tag @count(1..5)         # with cardinality hint
collaborators: <->User @count(0..10)
```

### 4.2 Reference Decorators

| Decorator                  | Meaning                                     |
|----------------------------|---------------------------------------------|
| `@cascade`                 | Delete cascades to children                  |
| `@restrict`               | Prevent delete if children exist              |
| `@set_null`               | Set to null on parent delete                  |
| `@eager`                  | Include in default fetch                      |
| `@lazy`                   | Only fetch on explicit request                |

### 4.3 Polymorphic References

```yaml
commentable: ->Post | ->Photo | ->Video
# generates a (type, id) pair
```

---

## 5. Entity-Level Decorators

Applied to the entity as a whole using a `_meta` field or inline annotation.

```yaml
entities:
  AuditLog:
    _meta: @readonly @immutable @paginated(100)
    id: uuid @primary
    actor: ->User
    action: enum(create, update, delete)
    entity_type: string
    entity_id: uuid
    timestamp: datetime @auto
    diff: any?
```

| Entity Decorator       | Meaning                                       |
|------------------------|-----------------------------------------------|
| `@readonly`            | No mutation tools generated                    |
| `@immutable`           | Can be created but never updated or deleted    |
| `@cacheable(ttl)`      | Hint for caching layer; ttl in seconds         |
| `@paginated(size)`     | Default page size for list operations          |
| `@timestamps`          | Auto-add `created_at` and `updated_at` fields |
| `@soft_delete`         | Auto-add `deleted_at`, filter from queries     |
| `@versioned`           | Auto-add `version` field, optimistic locking   |

---

## 6. Cross-Entity Rules

```yaml
rules:
  # Temporal ordering
  - Order.shipped_at > Order.placed_at
  - Order.delivered_at > Order.shipped_at

  # Referential consistency
  - Order.shipping_address.country == Order.customer.address.country
    @probability(0.85)            # true 85% of the time

  # Conditional existence
  - if Order.status == "shipped" then Order.tracking_number != null

  # Aggregate constraints
  - Customer.lifetime_value == sum(Customer.orders.total)

  # Uniqueness across fields
  - unique(Employee.department, Employee.badge_number)

  # Cardinality constraints
  - count(Team.members) >= 2
  - count(Project.leads) in 1..3
```

**Rule modifiers:**

| Modifier              | Meaning                                       |
|-----------------------|-----------------------------------------------|
| `@probability(p)`    | Rule holds with probability p during generation|
| `@strict`            | Rule must always hold (validation constraint)  |
| `@warn`              | Log warning if violated, don't fail            |

---

## 7. Reusable Types

```yaml
types:
  Address:
    line1: address.street
    line2: string? @null_rate(0.6)
    city: address.city
    state: address.state
    zip: address.zip
    country: address.country @default("US")

  Money:
    amount: decimal(10,2) @range(0..999999)
    currency: enum(USD, EUR, GBP, JPY) @dist(60, 20, 10, 10)

  DateRange:
    start: date
    end: date @after(start)

entities:
  Customer:
    billing_address: Address
    shipping_address: Address
    credit: Money
```

---

## 8. Shared Enums

```yaml
enums:
  Priority: [critical, high, medium, low]
  Status: [draft, active, archived]
  Continent:
    - value: NA
      label: "North America"
      weight: 25
    - value: EU
      label: "Europe"
      weight: 30
    - value: AS
      label: "Asia"
      weight: 35
    - value: other
      label: "Other"
      weight: 10
```

Weighted enum syntax provides inline distribution without a separate `@dist`.

### 8.1 Variant Descriptions

Each variant can carry an optional `description` for documentation purposes. Descriptions appear in `inspect` output and help collaborators understand domain-specific codes.

```yaml
enums:
  MaterialType:
    - value: STDMATERIAL
      description: "Standard direct materials for production"
    - value: STDSERVICE
      description: "Standard services (consulting, maintenance, etc.)"
    - value: NONSTD
      description: "Non-standard or one-off purchases"
    - value: MRO
      description: "Maintenance, Repair, and Operations supplies"
```

All variant fields (`label`, `weight`, `description`) are optional and can be combined freely:

```yaml
enums:
  Region:
    - value: NA
      label: "North America"
      weight: 25
      description: "US, Canada, and Mexico operations"
    - value: APAC
      label: "Asia-Pacific"
      weight: 35
      description: "Includes ANZ, SEA, and Greater China"
```

---

## 9. Tool Inference Rules

Tools are auto-generated from entity definitions. The inference engine follows these rules.

### 9.1 Default Tool Surface per Entity

Every non-`@readonly` entity produces:

| Tool       | Verb     | Description                              |
|------------|----------|------------------------------------------|
| `list`     | `GET`    | Paginated list with filter/sort          |
| `get`      | `GET`    | Fetch single record by primary key       |
| `create`   | `POST`   | Create new record                        |
| `update`   | `PATCH`  | Partial update of mutable fields         |
| `delete`   | `DELETE` | Remove record (or soft-delete)           |

`@readonly` entities get only `list` and `get`.
`@immutable` entities get `list`, `get`, and `create`.

### 9.2 Field → Tool Input/Output Mapping

| Decorator        | In `create` | In `update` | In `list` response | In `get` response |
|------------------|-------------|-------------|---------------------|--------------------|
| `@auto`          | ✗           | ✗           | ✓                   | ✓                  |
| `@primary`       | ✗           | ✗ (used as key) | ✓               | ✓                  |
| `@immutable`     | ✓           | ✗           | ✓                   | ✓                  |
| `@derived`       | ✗           | ✗           | ✓                   | ✓                  |
| `@hidden`        | ✓           | ✓           | ✗                   | ✓                  |
| `@sensitive`     | ✓           | ✓           | ✗ (masked)          | ✗ (masked)         |
| `@optional`      | optional    | optional    | ✓                   | ✓                  |
| `@default(v)`    | optional    | optional    | ✓                   | ✓                  |
| (none)           | required    | optional    | ✓                   | ✓                  |

### 9.3 Filter & Sort Inference

The `list` tool automatically includes:

**Filters for:**
- All `@filterable` fields
- All `@index` fields
- All enum fields
- All reference (`->`) fields
- All `bool` fields
- `datetime` / `date` fields (range filters: `_after`, `_before`)

**Sort by:**
- All `@sortable` fields
- `@primary` field
- `datetime` / `date` fields
- Numeric fields with `@index`

**Search:**
- All `@searchable` fields contribute to a `q` full-text parameter

### 9.4 Relationship-Derived Tools

References automatically produce sub-tools:

```yaml
# Given:
Customer:
  orders: [Order] @count(0..50)

# Inferred tool:
Customer.orders → list Orders filtered by customer_id
# Equivalent to: Order.list(customer=<id>)
```

Many-to-many references produce link/unlink tools:

```yaml
# Given:
Post:
  tags: <->Tag @count(1..5)

# Inferred tools:
Post.tags.list    → list tags for a post
Post.tags.link    → add tag to post
Post.tags.unlink  → remove tag from post
```

### 9.5 Tool Override Syntax

Override any inferred behavior:

```yaml
tools:
  Customer:
    list:
      filters: [tier, created_at, email]   # explicit filter set
      sorts: [name, created_at]            # explicit sort set
      page_size: 25                         # override default
      max_page_size: 100
    create:
      required: [name, email]              # override required fields
      optional: [tier, phone]
      defaults: { tier: free }
    update:
      mutable: [name, email, tier, phone]  # explicit mutable set
      immutable: [created_at]
    delete:
      strategy: soft                        # soft | hard | disabled
    # disable a tool entirely
    # delete: disabled

  Order:
    create: disabled                        # orders created via workflow only
    update:
      mutable: [status]                     # only status can change
```

---

## 10. Volume & Generation Control

```yaml
volume:
  Customer: 1000
  Product: 200
  Order: 5000                    # can also use range: 4000..6000
  LineItem: ~                    # inferred from Order × @count

generation:
  seed: 42                       # deterministic output
  locale: en-US                  # affects names, addresses, phone formats
  locales:                       # multi-locale distribution
    en-US: 60
    es-MX: 20
    ja-JP: 10
    de-DE: 10
  null_strategy: realistic       # realistic | never | sparse
  id_format: uuid                # uuid | sequential | cuid | ulid
  date_format: iso8601           # output format hint
  currency_format: decimal       # decimal | integer_cents
```

---

## 11. Coherence Groups

Coherence groups ensure correlated fields are generated together so they make sense as a unit.

```yaml
entities:
  Customer:
    name: person.full
    email: email @from(name)           # implicit coherence
    phone: phone @locale(address)      # phone format matches address locale
    address: Address

  Employee:
    _coherence:
      identity: [first_name, last_name, email, username]
      location: [office, timezone, phone]
      role: [department, title, level, salary]

    first_name: person.first
    last_name: person.last
    email: email @from(first_name, last_name) @pattern("{first}.{last}@company.com")
    username: person.username @from(first_name, last_name)
    department: job.department
    title: job.title @from(department)  # title matches department
    level: enum(junior, mid, senior, staff, principal) @correlated(salary, r=0.9)
    salary: currency.usd @range(45000..350000)
    office: enum(NYC, SF, LON, TYO)
    timezone: timezone @from(office)    # timezone matches office location
    phone: phone @from(office)          # phone area code matches office
```

---

## 12. Complete Example

```yaml
domain: project_management
version: 0.1.0
seed: 42
locale: en-US

volume:
  Organization: 5
  User: 200
  Project: 50
  Task: 2000
  Comment: 5000

enums:
  Priority: [critical, high, medium, low]
  TaskStatus: [backlog, todo, in_progress, review, done, cancelled]

types:
  Timestamps:
    created_at: datetime @auto @immutable
    updated_at: datetime @auto

entities:
  Organization:
    _meta: @timestamps
    id: uuid @primary
    name: company.name @unique
    slug: slug @derived(slug(name)) @unique
    plan: enum(free, team, business, enterprise) @dist(40, 30, 20, 10)
    member_count: int @derived(count(members))
    members: [User] @count(5..80)

  User:
    _meta: @timestamps @soft_delete
    id: uuid @primary
    org: ->Organization
    name: person.full
    email: email @unique @from(name)
    avatar: url.avatar? @null_rate(0.3)
    role: enum(admin, manager, member, viewer) @dist(5, 15, 70, 10)
    last_login: datetime? @range(now-90d..now) @null_rate(0.1)
    assigned_tasks: [Task] @lazy

  Project:
    _meta: @timestamps @soft_delete
    id: uuid @primary
    org: ->Organization
    name: string @len(3..60) @searchable
    key: string @pattern("{AAA}") @unique @immutable
    description: text.paragraph? @null_rate(0.2)
    lead: ->User @filterable
    status: enum(planning, active, paused, completed, archived) @dist(10, 50, 10, 20, 10)
    start_date: date? @range(now-1y..now+3m)
    target_date: date? @after(start_date) @within(1y, start_date)
    task_count: int @derived(count(tasks))
    tasks: [Task] @count(10..80)

  Task:
    _meta: @timestamps
    id: uuid @primary
    project: ->Project
    key: string @derived(concat(project.key, "-", seq)) @unique
    title: string @len(5..120) @searchable
    description: text.paragraph? @null_rate(0.4)
    status: TaskStatus @dist(10, 15, 20, 15, 35, 5)
    priority: Priority @dist(5, 15, 50, 30)
    assignee: ->User? @null_rate(0.15)
    reporter: ->User
    estimate_hours: float? @range(0.5..40) @dist(lognormal, μ=1.5, σ=0.8) @null_rate(0.3)
    due_date: date? @after(project.start_date) @before(project.target_date) @null_rate(0.4)
    parent: ->self? @null_rate(0.7)
    labels: <->Label @count(0..4)
    comments: [Comment] @count(0..15)

  Label:
    id: uuid @primary
    project: ->Project
    name: string @len(2..30) @values(bug, feature, improvement, docs, tech-debt, ux, perf)
    color: color.hex

  Comment:
    _meta: @timestamps
    id: uuid @primary
    task: ->Task
    author: ->User
    body: text.paragraph @len(10..500)
    edited: bool @dist(90, 10)

rules:
  - Task.assignee.org == Task.project.org       @strict
  - Task.reporter.org == Task.project.org       @strict
  - Comment.author.org == Comment.task.project.org @strict
  - if Task.status == "done" then Task.assignee != null
  - Task.updated_at >= Task.created_at
  - Comment.created_at >= Comment.task.created_at

tools:
  Task:
    list:
      filters: [project, status, priority, assignee, labels, due_date]
      sorts: [created_at, updated_at, priority, due_date]
      page_size: 50
    update:
      mutable: [title, description, status, priority, assignee, estimate_hours, due_date, parent]
  Comment:
    delete: disabled
```

---

## 13. Formal Grammar (EBNF Sketch)

```ebnf
document      = header , entities , [ enums ] , [ types ] , [ rules ] , [ tools ] ;
header        = "domain:" , identifier , { header_field } ;
header_field  = ( "version:" | "seed:" | "locale:" ) , value ;

entities      = "entities:" , { entity_def } ;
entity_def    = entity_name , ":" , { field_def } ;
field_def     = field_name , ":" , type_expr , { decorator } ;

type_expr     = primitive | semantic | enum_inline | reference | compound | named_type ;
primitive     = "string" | "int" | "float" | "bool" | "datetime" | "date"
              | "time" | "duration" | "uuid" | "bytes" | "decimal" | "any" ;
semantic      = namespace , "." , tag , [ "(" , params , ")" ] ;
enum_inline   = "enum(" , value , { "," , value } , ")" ;
reference     = "->" , entity_name , [ "?" ]
              | "->self" , [ "?" ]
              | "->[" , entity_name , "]"
              | "<->" , entity_name ;
compound      = "[" , type_expr , "]"
              | "{" , type_expr , ":" , type_expr , "}"
              | "(" , type_expr , { "," , type_expr } , ")"
              | type_expr , "?"
              | type_expr , "|" , type_expr ;

decorator     = "@" , decorator_name , [ "(" , decorator_args , ")" ] ;
decorator_name = identifier ;
decorator_args = value | range | distribution | expression ;
range         = value , ".." , value ;
distribution  = dist_name , { "," , param_assign } ;
expression    = term , { operator , term } ;

rules         = "rules:" , { rule_def } ;
rule_def      = "-" , expression , { rule_modifier } ;
rule_modifier = "@" , ( "probability" | "strict" | "warn" ) , [ "(" , value , ")" ] ;
```

---

## 14. Design Principles

1. **Semantic over structural.** A field tagged `person.full` carries more information than `string @len(5..50)`. Prefer semantic types wherever they exist.

2. **Defaults are realistic.** Without any decorators, generators should produce plausible data. Decorators *refine*, not *enable*.

3. **Derivation over duplication.** If a value can be computed from other fields, use `@derived`. This keeps the schema as the single source of truth and prevents contradictions in generated data.

4. **Coherence over independence.** Fields within an entity should make sense together. A person in Tokyo should have a Japanese phone number. Coherence groups and `@from` decorators encode these relationships.

5. **Tools are a projection of schema.** The tool surface is a view over the entity graph. Overrides are for policy (who can do what), not for structure.

6. **Constraints are for generators too.** Rules aren't just validation — they guide generation. A rule like `order.shipped > order.placed` tells the generator to produce temporally consistent data.

7. **Progressive disclosure.** A minimal entity definition with just field names and semantic types should produce useful data. Each decorator layer adds precision without breaking the base case.