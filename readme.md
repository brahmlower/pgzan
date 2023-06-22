# pgzan

Do you use Postgres [row level security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html), but find it difficult to enforce and maintain your applications access control needs? Well fret no more! `pgzan` is a work in progress extension for Postgres that aims to bring the flexibility and expressivness of Googles worldclass [Zanzibar](https://research.google/pubs/pub48190/) authorization system right down to your postgres row policies.

That's right! With `pgzan` you can leverage the benefits of a world class ACL on a row-by-row basis with minimal additional dev work!

## ⚠️ Work in Progress ⚠️

This cursed project is a work in progress. Do not bring it remotely close to your production databases.

## The Dream 🙌

Okay so lets say you're running a multi-tenant product and have a table of important info, with references to an account that owns a particular record:

```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE accounts (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "name" VARCHAR(256) NOT NULL,
);

CREATE TABLE failed_projects (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "account" UUID REFERENCES accounts(id) NOT NULL,
    "name" VARCHAR(256) NOT NULL,
);
```

Now when listing the projects table, how do you control which projects are returned for a given query? If you said "a filter", then that's some old-age think'n bud. Postgres row level policies are the real solution here 💪 "But my fortune 50 enterprise customer needs me to support a complex RBAC solution" well shit we can't be turning down those fat oligarchy dollars now can we! So lets do this right and bring `pgzan` into the mix.

**First:** Download `pgzan`! How? I dunno yet, this shit ain't hit the prime time yet so you'll just have to compile it from source like your grandpa. Gander at the [development](#Development) section for compilation instructions.

**Second:** Now install it! How? Not sure- check back later? For now just run `cargo pgrx run` I guess, then quit out of the shell. Sheems like that puts the code up in the db 🤷

**Third:** Enable the extension:

```sql
CREATE EXTENSION IF NOT EXISTS "pgzan";
```

**Fourth:** Time to rock and roll! Enable that sweet sweet RLS on your table and define the policy as something vaguely similar to the following.

```sql
BEGIN;

ALTER TABLE failed_projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE failed_projects FORCE ROW LEVEL SECURITY;

CREATE POLICY failed_projects_policy ON failed_projects
USING (
    SELECT pgzan_check(
        current_setting('authed_failed_projects.account_id')::UUID,
        "manager"
    )
)
WITH CHECK (
    SELECT pgzan_check(
        current_setting('authed_failed_projects.account_id')::UUID,
        "manager"
    )
);

COMMIT;
```

**Fith:** Okay final step before locking down that sales contract- we need to set the context for the connection. Notice we're using the `authed_failed_projects.account_id` setting to tell pgzan who we're evaluating permissions for, but we haven't declared a value for that anywhere. When creating your db connection just make sure you set that value before making any queries and you'll be golden:

```sql
SET SESSION authed_failed_projects.account_id = "a7f3da20-e862-48bc-b2cd-49b42894eef5"
```

**Sixth:** Hell yeah, we've made the investors proud and fulfilled our capitalistic duties 🫡 Now let's go get blasted 🍻

## Development

This project depends on [pgrx](https://github.com/tcdi/pgrx)- follow their [system requirements](https://github.com/tcdi/pgrx#system-requirements) section before installing `cargo-pgrx`.

Initial dependency setup:

```shell
$ cargo install --locked cargo-pgrx
```

Building and running the extension (will drop you into a pgsql shell):

```shell
$ cargo pgrx run
```

You can test the current functionality by running `pgzan_check` with a json blob with an `id` and `role` field. The role may be either "manager" or "readonly". The function is hardcoded to evaluate the role against an "update" operation which is only permitted by the "manager" role, so you can see how the function response would feasibly connect back to the row level policy:

```sql
pgzan=# CREATE EXTENSION pgzan;
CREATE EXTENSION
pgzan=# SELECT pgzan_check('{"id": "07b30b3a-8da9-465e-96ef-4054f870cd8a", "role": "readonly"}');
 pgzan_check
-------------
 f
(1 row)

pgzan=# SELECT pgzan_check('{"id": "07b30b3a-8da9-465e-96ef-4054f870cd8a", "role": "manager"}');
 pgzan_check
-------------
 t
(1 row)
```

Significant development is needed to make the ACL customizable (it's hardcoded right now) and to properly connect the necessary session context values to the row level policy.

Though lots of work is needed, the current state of the project demonstrates that this _is_ possible. Let's build something beautiful; let's build an abomination ✨
