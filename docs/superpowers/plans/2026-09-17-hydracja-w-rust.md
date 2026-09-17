# Hydracja po stronie Rusta — plan implementacji

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Przenieść decyzje hydracyjne z JavaScriptu do Rusta — JS zostaje wykonawcą komend, który buduje snapshot DOM i stosuje komendy, nie podejmując żadnej decyzji o dopasowaniu.

**Architecture:** Rust pyta przeglądarkę o snapshot DOM nowym wariantem `CommandForBrowser::DomSnapshotGet`. Zamiast wysłać bufor komend zebrany w czasie mountu, buduje z niego indeks drzewa docelowego, uzgadnia go ze snapshotem i emituje zredukowany strumień: adopcje istniejących węzłów plus same różnice. Dwie nowe komendy (`NodeAdopt`, `SnapshotRemove`) adresują węzły przeglądarki przez indeks w snapshocie.

**Tech Stack:** Rust 2024 (workspace `vertigo`, `vertigo-cli`, `vertigo-macro`), TypeScript 6 budowany Rollupem do `wasm_run.js`, `AutoJsJson` jako format wymiany, własny format druciarski dla strumienia DOM.

**Spec:** `docs/superpowers/specs/2026-09-17-hydracja-w-rust-design.md`

## Global Constraints

- **Wszystkie komentarze w kodzie — w tym doc comments — piszemy po angielsku.** Całe
  repozytorium jest po angielsku, a `vertigo` to publiczna skrzynka, więc doc comments
  jadą na docs.rs. Bloki kodu w tym planie mają komentarze po polsku, bo plan jest
  notatką robocza pisaną po polsku: **przy przepisywaniu do kodu tłumacz je na
  angielski**, zachowując treść i to, że tłumaczą *dlaczego*, a nie *co*. Polski
  zostaje w tym planie i w specyfikacji.
- `unwrap_used` i `expect_used` są w workspace ustawione na `deny` (`Cargo.toml`, `[workspace.lints.clippy]`). W kodzie produkcyjnym **i w testach** używaj `match`, `let ... else` albo `panic!` z komunikatem.
- Tagi formatu druciarskiego są **append-only**: numer raz użyty nie może dostać innego znaczenia (`crates/vertigo/src/dev/command_wire.rs`, komentarz nad `mod tag`).
- Identyfikator 0 jest sentinelem „brak węzła odniesienia" w `InsertBefore`. Żaden prawdziwy `DomId` nie może być zerem (`debug_assert` w `write_id`).
- `DomId` 1, 2 i 3 są zarezerwowane dla `html`, `head` i `body` (`DomId::from_name`). `MapNodes.set` je ignoruje, `MapNodes.getAnyOption` rozwiązuje je dynamicznie — nigdy nie są adoptowane.
- Komendy DOM nie przechodzą przez `AutoJsJson`, tylko przez format pozycyjny z `command_wire.rs`. Pozostałe komunikaty przechodzą przez `AutoJsJson`.
- Testy jednostkowe uruchamiane są na hoście, nie w wasmie. `crates/vertigo/src/external_api.rs` dostarcza wtedy atrapę `safe_dom_access`, która odpowiada na każdy wariant `CommandForBrowser`.
- Po każdym teście, który montuje aplikację, trzeba zwolnić drzewo przez `drop(get_driver().take_root())` — inaczej porzucenie `DomNode` przy zamykaniu wątku sięga do już zwolnionego store'u i przerywa proces. Wzór: `crates/vertigo/src/tests/mount_batching.rs`.
- Komendy: testy Rusta `cargo test --all-features`, testy JS `npm run test`, budowa bundla JS `npx rollup -c`, clippy `cargo clippy --locked -p vertigo --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings`.

---

## File Structure

**Nowe pliki**

| Plik | Odpowiedzialność |
|---|---|
| `crates/vertigo/src/driver_module/hydration/mod.rs` | Korzeń modułu; publiczne `reconcile`, `split_buffer`, typy snapshotu. |
| `crates/vertigo/src/driver_module/hydration/snapshot.rs` | `DomSnapshot`, `SnapshotNode`, `SnapshotAttr` — format odpowiedzi z przeglądarki. |
| `crates/vertigo/src/driver_module/hydration/target_tree.rs` | Indeks drzewa docelowego zbudowany z bufora komend plus podział bufora. |
| `crates/vertigo/src/driver_module/hydration/matcher.rs` | Dopasowanie do snapshotu i emisja strumienia wynikowego. |
| `crates/vertigo/src/driver_module/hydration/report.rs` | `HydrationReport` i jego publikacja na `window`. |
| `crates/vertigo/src/driver_module/api/api_dom_snapshot.rs` | Pobranie snapshotu z przeglądarki; punkt wstrzyknięcia atrapy w testach. |
| `crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.ts` | Przejście po DOM i budowa snapshotu. |
| `crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.test.ts` | Testy buildera snapshotu na atrapie DOM. |

**Modyfikowane pliki**

| Plik | Zmiana |
|---|---|
| `crates/vertigo/src/dev/command.rs` | `CommandForBrowser::DomSnapshotGet`; `DriverDomCommand::NodeAdopt` i `SnapshotRemove`. |
| `crates/vertigo/src/dev/command_wire.rs` | Tagi 14 i 15, kodowanie, dekodowanie, fixture. |
| `crates/vertigo/src/driver_module/api/api_browser_command.rs` | Metoda `dom_snapshot_get`. |
| `crates/vertigo/src/driver_module/api/mod.rs` | Eksport `api_dom_snapshot`. |
| `crates/vertigo/src/driver_module/mod.rs` | Rejestracja modułu `hydration`. |
| `crates/vertigo/src/driver_module/dom.rs` | Tryb hydracji, `arm_hydration`, `flush_hydration`, klasyfikacja `SnapshotRemove` przy sortowaniu. |
| `crates/vertigo/src/exports.rs` | `mount` uzbraja tryb i kończy `flush_hydration`. |
| `crates/vertigo/src/external_api.rs` | Gałąź dla `DomSnapshotGet`. |
| `crates/vertigo-cli/src/serve/server_state.rs` | Gałąź dla `DomSnapshotGet`. |
| `crates/vertigo-cli/src/serve/html/element.rs` | Ignorowanie `NodeAdopt` i `SnapshotRemove` przy replayu. |
| `crates/vertigo/src/driver_module/src_js/api/api.ts` | Obsługa `DomSnapshotGet`. |
| `crates/vertigo/src/driver_module/src_js/api/command/dom/dom.ts` | Obsługa dwóch nowych komend; usunięcie gałęzi hydracji. |
| `crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.ts` | Tagi 14 i 15. |
| `crates/vertigo/src/driver_module/src_js/api/command/dom/map_nodes.ts` | Usunięcie maszynerii `initNodes`. |
| `crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.test.ts` | Fixture o dwa tagi dłuższy. |
| `rollup.test.config.mjs`, `package.json` | Wypisanie `hydration.test`, wpisanie `snapshot.test`. |
| `crates/vertigo/src/tests/mod.rs` | Rejestracja modułu testów hydracji. |
| `crates/vertigo/src/tests/mount_batching.rs` | Asercje pod nową treść paczki. |
| `crates/vertigo/src/tests/dom_command_counts.rs` | Liczby komend przy mount. |
| `tests/demo/ssr.rs` | Nazwy pól raportu. |
| `docs/CHANGELOG.md` | Wpis. |

**Usuwane pliki**

- `crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.ts`
- `crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.test.ts`

---

### Task 1: Typy snapshotu i kanał pobrania

**Files:**
- Create: `crates/vertigo/src/driver_module/hydration/mod.rs`
- Create: `crates/vertigo/src/driver_module/hydration/snapshot.rs`
- Create: `crates/vertigo/src/driver_module/api/api_dom_snapshot.rs`
- Modify: `crates/vertigo/src/dev/command.rs`
- Modify: `crates/vertigo/src/driver_module/api/api_browser_command.rs`
- Modify: `crates/vertigo/src/driver_module/api/mod.rs`
- Modify: `crates/vertigo/src/driver_module/mod.rs`
- Modify: `crates/vertigo/src/external_api.rs`
- Modify: `crates/vertigo-cli/src/serve/server_state.rs`

**Interfaces:**
- Produces: `DomSnapshot { nodes: Vec<SnapshotNode>, head: Option<u32>, body: Option<u32> }`,
  `SnapshotNode::{Element { name: String, attrs: Vec<SnapshotAttr>, children: Vec<u32> }, Text { value: String }, Comment { value: String }}`,
  `SnapshotAttr { name: String, value: String }`,
  `DomSnapshot::node(&self, u32) -> Option<&SnapshotNode>`,
  `DomSnapshot::children(&self, u32) -> &[u32]`,
  `api_dom_snapshot() -> Rc<ApiDomSnapshot>` z `get(&self) -> Option<DomSnapshot>` i `#[cfg(test)] set_mock(&self, DomSnapshot)`.

- [ ] **Step 1: Napisz failujący test serializacji w tę i z powrotem**

Utwórz `crates/vertigo/src/driver_module/hydration/snapshot.rs` z samym blokiem testów na końcu (typy dopisujesz w kroku 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsJsonSerialize, dev::command::decode_json};

    fn sample() -> DomSnapshot {
        DomSnapshot {
            nodes: vec![
                SnapshotNode::Element {
                    name: "html".to_string(),
                    attrs: vec![SnapshotAttr {
                        name: "lang".to_string(),
                        value: "pl".to_string(),
                    }],
                    children: vec![1, 2],
                },
                SnapshotNode::Element {
                    name: "head".to_string(),
                    attrs: vec![],
                    children: vec![],
                },
                SnapshotNode::Element {
                    name: "body".to_string(),
                    attrs: vec![],
                    children: vec![3, 4],
                },
                SnapshotNode::Text {
                    value: "zażółć 🦀".to_string(),
                },
                SnapshotNode::Comment {
                    value: "a marker".to_string(),
                },
            ],
            head: Some(1),
            body: Some(2),
        }
    }

    #[test]
    fn a_snapshot_survives_the_json_round_trip() {
        let json = sample().to_json();

        let back = match decode_json::<DomSnapshot>(json) {
            Ok(value) => value,
            Err(err) => panic!("decode failed: {err}"),
        };

        assert_eq!(format!("{back:?}"), format!("{:?}", sample()));
    }

    #[test]
    fn children_of_a_non_element_is_empty() {
        let snapshot = sample();

        assert_eq!(snapshot.children(0), &[1, 2]);
        assert_eq!(snapshot.children(3), &[] as &[u32]);
        assert_eq!(snapshot.children(999), &[] as &[u32]);
    }
}
```

- [ ] **Step 2: Uruchom test, żeby sprawdzić, że nie kompiluje się**

Run: `cargo test -p vertigo --all-features snapshot`
Expected: FAIL — `cannot find type DomSnapshot in this scope`

- [ ] **Step 3: Dopisz typy nad blokiem testów**

W `crates/vertigo/src/driver_module/hydration/snapshot.rs`, przed `#[cfg(test)] mod tests`:

```rust
use vertigo_macro::AutoJsJson;

/// Stan DOM odczytany z przeglądarki, w kolejności pre-order.
///
/// Indeks w [`Self::nodes`] jest adresem węzła w protokole: komendy
/// [`NodeAdopt`](crate::dev::command::DriverDomCommand::NodeAdopt) i
/// [`SnapshotRemove`](crate::dev::command::DriverDomCommand::SnapshotRemove) odwołują się
/// nim do prawdziwego węzła, który js trzyma w tablicy zbudowanej przy tym samym przejściu.
/// Element 0 to `<html>`.
#[derive(AutoJsJson, Debug, Clone)]
pub struct DomSnapshot {
    pub nodes: Vec<SnapshotNode>,
    pub head: Option<u32>,
    pub body: Option<u32>,
}

#[derive(AutoJsJson, Debug, Clone)]
pub enum SnapshotNode {
    Element {
        /// `tagName` zmniejszone do małych liter. Dopasowanie po stronie rusta porównuje
        /// bez uwzględniania wielkości liter, co jest poprawne dla html i svg naraz i
        /// oszczędza przeniesienia tablicy `SVG_TAGS` do wasma.
        name: String,
        attrs: Vec<SnapshotAttr>,
        children: Vec<u32>,
    },
    Text {
        value: String,
    },
    Comment {
        value: String,
    },
}

#[derive(AutoJsJson, Debug, Clone)]
pub struct SnapshotAttr {
    pub name: String,
    pub value: String,
}

impl DomSnapshot {
    pub fn node(&self, index: u32) -> Option<&SnapshotNode> {
        self.nodes.get(index as usize)
    }

    pub fn children(&self, index: u32) -> &[u32] {
        match self.node(index) {
            Some(SnapshotNode::Element { children, .. }) => children.as_slice(),
            _ => &[],
        }
    }

    pub fn attrs(&self, index: u32) -> &[SnapshotAttr] {
        match self.node(index) {
            Some(SnapshotNode::Element { attrs, .. }) => attrs.as_slice(),
            _ => &[],
        }
    }
}
```

Utwórz `crates/vertigo/src/driver_module/hydration/mod.rs`:

```rust
mod snapshot;

pub use snapshot::{DomSnapshot, SnapshotAttr, SnapshotNode};
```

W `crates/vertigo/src/driver_module/mod.rs` dopisz `pub mod hydration;` obok pozostałych deklaracji modułów.

- [ ] **Step 4: Uruchom testy**

Run: `cargo test -p vertigo --all-features snapshot`
Expected: PASS (2 testy)

- [ ] **Step 5: Dopisz komendę i jej obsługę po stronie hostów**

W `crates/vertigo/src/dev/command.rs`, w `enum CommandForBrowser`, dopisz wariant obok `FetchCacheGet`:

```rust
    /// Prośba o stan DOM przeglądarki na potrzeby hydracji.
    ///
    /// Pytamy, zamiast czekać na wypchnięcie z js, żeby kolejność nie była umową: rust
    /// pobiera snapshot dokładnie wtedy, gdy jest mu potrzebny, czyli w
    /// `DriverDom::flush_hydration`. Odpowiedzią jest [`DomSnapshot`] albo `Null`, gdy nie
    /// ma DOM do zwrócenia (renderowanie serwerowe, testy na hoście).
    DomSnapshotGet,
```

W `crates/vertigo/src/driver_module/api/api_browser_command.rs`, obok `fetch_cache_get`:

```rust
    pub fn dom_snapshot_get(&self) -> JsJson {
        exec_command(CommandForBrowser::DomSnapshotGet)
    }
```

W `crates/vertigo/src/external_api.rs` dopisz `DomSnapshotGet` do listy wariantów zwracających `JsJson::Null` (ta sama gałąź, w której jest już `DomBulkUpdate`).

W `crates/vertigo-cli/src/serve/server_state.rs`, w `handle_command`, dopisz gałąź:

```rust
                    CommandForBrowser::DomSnapshotGet => JsJson::Null,
```

- [ ] **Step 6: Napisz failujący test pobierania snapshotu**

Utwórz `crates/vertigo/src/driver_module/api/api_dom_snapshot.rs` z samym blokiem testów:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver_module::hydration::{DomSnapshot, SnapshotNode};

    #[test]
    fn without_a_browser_there_is_no_snapshot() {
        assert!(api_dom_snapshot().get().is_none());
    }

    #[test]
    fn a_mocked_snapshot_is_returned_as_is() {
        api_dom_snapshot().set_mock(DomSnapshot {
            nodes: vec![SnapshotNode::Element {
                name: "html".to_string(),
                attrs: vec![],
                children: vec![],
            }],
            head: None,
            body: None,
        });

        let Some(snapshot) = api_dom_snapshot().get() else {
            panic!("the mocked snapshot should come back");
        };

        assert_eq!(snapshot.nodes.len(), 1);
    }
}
```

- [ ] **Step 7: Uruchom test, żeby sprawdzić, że nie kompiluje się**

Run: `cargo test -p vertigo --all-features api_dom_snapshot`
Expected: FAIL — `cannot find function api_dom_snapshot in this scope`

- [ ] **Step 8: Dopisz implementację nad blokiem testów**

Wzoruj się na `crates/vertigo/src/driver_module/api/api_fetch.rs` — ma dokładnie ten sam układ atrapy pod `#[cfg(test)]`.

```rust
use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    JsJson,
    dev::command::decode_json,
    driver_module::{api::api_browser_command, hydration::DomSnapshot},
};

#[cfg(test)]
use crate::dev::ValueMut;

#[store]
pub fn api_dom_snapshot() -> Rc<ApiDomSnapshot> {
    Rc::new(ApiDomSnapshot {
        #[cfg(test)]
        mock: ValueMut::new(None),
    })
}

/// Stan DOM przeglądarki, pobierany raz, w czasie montowania aplikacji.
///
/// Osobny store, a nie metoda na `DriverDom`, z tego samego powodu, dla którego osobny jest
/// `api_fetch_cache`: aplikacja, która nigdy nie jest hydratowana, nie wciąga dekodera
/// snapshotu do wasma.
pub struct ApiDomSnapshot {
    #[cfg(test)]
    mock: ValueMut<Option<Rc<DomSnapshot>>>,
}

impl ApiDomSnapshot {
    #[cfg(test)]
    pub fn set_mock(&self, snapshot: DomSnapshot) {
        self.mock.set(Some(Rc::new(snapshot)));
    }

    pub fn get(&self) -> Option<DomSnapshot> {
        #[cfg(test)]
        if let Some(mock) = self.mock.get() {
            return Some((*mock).clone());
        }

        let json = api_browser_command().dom_snapshot_get();

        if let JsJson::Null = json {
            return None;
        }

        match decode_json::<DomSnapshot>(json) {
            Ok(snapshot) => Some(snapshot),
            Err(err) => {
                log::error!("dom snapshot decode error = {err}");
                None
            }
        }
    }
}
```

W `crates/vertigo/src/driver_module/api/mod.rs` dopisz moduł i reeksport obok `api_fetch_cache`:

```rust
mod api_dom_snapshot;
pub use api_dom_snapshot::api_dom_snapshot;
```

- [ ] **Step 9: Uruchom testy**

Run: `cargo test -p vertigo --all-features api_dom_snapshot snapshot`
Expected: PASS

Uwaga: testy w jednym pliku dzielą store'y w obrębie wątku, a `#[store]` jest thread-local. `cargo test` domyślnie zrównolegla po wątkach, więc atrapa z jednego testu nie przecieka do drugiego.

- [ ] **Step 10: Sprawdź, że całość się kompiluje i przechodzi clippy**

Run: `cargo test --all-features && cargo clippy --locked -p vertigo --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings`
Expected: PASS

- [ ] **Step 11: Commit**

```bash
git add crates/vertigo/src/driver_module/hydration crates/vertigo/src/driver_module/api/api_dom_snapshot.rs crates/vertigo/src/driver_module/api/mod.rs crates/vertigo/src/driver_module/mod.rs crates/vertigo/src/dev/command.rs crates/vertigo/src/driver_module/api/api_browser_command.rs crates/vertigo/src/external_api.rs crates/vertigo-cli/src/serve/server_state.rs
git commit -m "feat(hydration): snapshot types and the DomSnapshotGet channel"
```

---

### Task 2: Komendy NodeAdopt i SnapshotRemove w formacie druciarskim

**Files:**
- Modify: `crates/vertigo/src/dev/command.rs`
- Modify: `crates/vertigo/src/dev/command_wire.rs`
- Modify: `crates/vertigo-cli/src/serve/html/element.rs`
- Modify: pozostałe wyczerpujące `match`e po `DriverDomCommand`, które wskaże kompilator:
  `crates/vertigo/src/dev/inspect.rs`, `crates/vertigo/src/tests/dom_command_counts.rs`,
  `tests/dom-bench/src/counts.rs`
- Modify: `crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.ts`
- Modify: `crates/vertigo/src/driver_module/src_js/api/command/dom/dom.ts`
- Modify: `crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.test.ts`

**Interfaces:**
- Consumes: nic z Taska 1.
- Produces: `DriverDomCommand::NodeAdopt { id: DomId, snapshot: u32 }`, `DriverDomCommand::SnapshotRemove { snapshot: u32 }`; w TS `Tag.NodeAdopt = 14`, `Tag.SnapshotRemove = 15` oraz warianty `CommandType`: `{ NodeAdopt: { id: number, snapshot: number } }` i `{ SnapshotRemove: { snapshot: number } }`.

- [ ] **Step 1: Napisz failujący test przejścia przez format**

W `crates/vertigo/src/dev/command_wire.rs`, w `mod tests`, dopisz:

```rust
    /// Adopcja i usunięcie węzła snapshotu adresują węzeł liczbą porządkową z
    /// `DomSnapshot::nodes`, nie `DomId`. Zero jest tu poprawną wartością - `<html>` ma
    /// indeks 0 - więc w odróżnieniu od `write_id` nie ma tu sentinela.
    #[test]
    fn round_trips_the_snapshot_commands() {
        let commands = vec![
            DriverDomCommand::NodeAdopt {
                id: id(4),
                snapshot: 0,
            },
            DriverDomCommand::NodeAdopt {
                id: id(70000),
                snapshot: 300,
            },
            DriverDomCommand::SnapshotRemove { snapshot: 0 },
            DriverDomCommand::SnapshotRemove { snapshot: 1 << 20 },
        ];

        let decoded = decoded(&encode_dom_commands(&commands));

        assert_eq!(format!("{decoded:?}"), format!("{commands:?}"));
    }
```

- [ ] **Step 2: Uruchom test, żeby sprawdzić, że nie kompiluje się**

Run: `cargo test -p vertigo --all-features round_trips_the_snapshot_commands`
Expected: FAIL — `no variant named NodeAdopt found for enum DriverDomCommand`

- [ ] **Step 3: Dopisz warianty i kodowanie**

W `crates/vertigo/src/dev/command.rs`, w `enum DriverDomCommand`:

```rust
    /// Wiąże istniejący węzeł przeglądarki z identyfikatorem vertigo.
    ///
    /// Kierunek jest istotny: kluczem zostaje `DomId`, a pod niego podstawiany jest węzeł
    /// przeglądarki. Dzięki temu wszystko, co przyjdzie później - `SetAttr`, `UpdateText`,
    /// `CallbackAdd`, patche z subskrypcji - trafia w niezmienione identyfikatory i nie
    /// wymaga żadnej wiedzy o hydracji.
    NodeAdopt { id: DomId, snapshot: u32 },
    /// Usuwa węzeł snapshotu, którego nie adoptowano.
    ///
    /// `RemoveNode` nie da się tu użyć: resztki po renderowaniu serwerowym nie mają żadnego
    /// `DomId`, bo nigdy nie zostały zarejestrowane. Usunięcie węzła zabiera w DOM całe jego
    /// poddrzewo, więc dla odrzuconego poddrzewa wystarcza jedna komenda na jego korzeń.
    SnapshotRemove { snapshot: u32 },
```

W tym samym pliku, w `DriverDomCommand::is_event()` (metoda decydująca, co `sort_commands` przesuwa na koniec), dopisz `SnapshotRemove` do grupy usuwającej — obok `RemoveNode`, `RemoveText` i `RemoveComment`. `NodeAdopt` do niej **nie** należy.

W `crates/vertigo/src/dev/command_wire.rs`, w `mod tag`:

```rust
    pub const NODE_ADOPT: u8 = 14;
    pub const SNAPSHOT_REMOVE: u8 = 15;
```

W `write_command`, w gałęziach `match command`:

```rust
        DriverDomCommand::NodeAdopt { id, snapshot } => {
            out.push(tag::NODE_ADOPT);
            write_id(out, *id);
            write_varint(out, u64::from(*snapshot));
        }
        DriverDomCommand::SnapshotRemove { snapshot } => {
            out.push(tag::SNAPSHOT_REMOVE);
            write_varint(out, u64::from(*snapshot));
        }
```

W `read_command`:

```rust
        tag::NODE_ADOPT => DriverDomCommand::NodeAdopt {
            id: cursor.id()?,
            snapshot: cursor.varint()? as u32,
        },
        tag::SNAPSHOT_REMOVE => DriverDomCommand::SnapshotRemove {
            snapshot: cursor.varint()? as u32,
        },
```

W `crates/vertigo-cli/src/serve/html/element.rs`, w `AllElements::feed`, dopisz oba warianty do gałęzi ignorowanej przy renderowaniu serwerowym — tam, gdzie już ignorowane są `CallbackAdd` i `CallbackRemove`. Renderowanie serwerowe nigdy ich nie zobaczy (host odpowiada `Null` na `DomSnapshotGet`), ale `match` jest wyczerpujący.

- [ ] **Step 4: Uruchom test**

Run: `cargo test -p vertigo --all-features round_trips_the_snapshot_commands`
Expected: PASS

- [ ] **Step 5: Rozszerz międzyjęzykowy fixture**

W `crates/vertigo/src/dev/command_wire.rs`, w `fn fixture_commands()`, dopisz na koniec wektora:

```rust
            DriverDomCommand::NodeAdopt {
                id: id(4),
                snapshot: 0,
            },
            DriverDomCommand::SnapshotRemove { snapshot: 300 },
```

Stała `FIXTURE` jest zapisanym wprost ciągiem bajtów i teraz nie będzie się zgadzać. Nie licz tych bajtów ręcznie — weź je z komunikatu błędu.

Run: `cargo test -p vertigo --all-features matches_the_cross_language_fixture`
Expected: FAIL. W komunikacie `assert_eq!` po stronie `left` jest nowe kodowanie. Przepisz je do `const FIXTURE`.

- [ ] **Step 6: Uruchom test, żeby potwierdzić nowy fixture**

Run: `cargo test -p vertigo --all-features command_wire`
Expected: PASS

- [ ] **Step 7: Dopisz tagi po stronie TypeScriptu**

W `crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.ts`, w stałych `Tag`:

```ts
    NodeAdopt: 14,
    SnapshotRemove: 15,
```

W `decodeCommands` dopisz gałęzie dekodera obiektowego (kolejność pól musi odpowiadać `write_command`: dla `NodeAdopt` najpierw id, potem indeks):

```ts
            case Tag.NodeAdopt: {
                const id = cursor.varint();
                const snapshot = cursor.varint();
                commands.push({ NodeAdopt: { id, snapshot } });
                break;
            }
            case Tag.SnapshotRemove: {
                const snapshot = cursor.varint();
                commands.push({ SnapshotRemove: { snapshot } });
                break;
            }
```

W `crates/vertigo/src/driver_module/src_js/api/command/dom/dom.ts`, w typie `CommandType`, dopisz dwa warianty:

```ts
} | {
    NodeAdopt: { id: number, snapshot: number }
} | {
    SnapshotRemove: { snapshot: number }
```

Gorąca ścieżka `update` dostanie obsługę tych tagów w Tasku 7 — tutaj chodzi tylko o to, żeby fixture przechodził w obie strony.

- [ ] **Step 8: Przepisz bajty fixture'u do testu TypeScriptu**

W `crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.test.ts` podmień tablicę bajtów na tę samą, którą wpisałeś do `const FIXTURE`, i dopisz do oczekiwanej listy komend:

```ts
    { NodeAdopt: { id: 4, snapshot: 0 } },
    { SnapshotRemove: { snapshot: 300 } },
```

- [ ] **Step 9: Uruchom testy JS**

Run: `npm install && npm run test`
Expected: PASS — bajty po obu stronach są identyczne

- [ ] **Step 10: Commit**

```bash
git add crates/vertigo/src/dev/command.rs crates/vertigo/src/dev/command_wire.rs crates/vertigo-cli/src/serve/html/element.rs crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.ts crates/vertigo/src/driver_module/src_js/api/command/dom/dom.ts crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.test.ts
git commit -m "feat(hydration): NodeAdopt and SnapshotRemove on the wire"
```

---

### Task 3: Indeks drzewa docelowego z bufora komend

**Files:**
- Create: `crates/vertigo/src/driver_module/hydration/target_tree.rs`
- Modify: `crates/vertigo/src/driver_module/hydration/mod.rs`

**Interfaces:**
- Consumes: `DriverDomCommand` (Task 2).
- Produces: `TargetKind::{Element { name: StaticString }, Text { value: String }, Comment { value: String }}`,
  `TargetNode { kind: TargetKind, attrs: BTreeMap<StaticString, String>, children: Vec<DomId> }`,
  `TargetTree` z `get(&self, DomId) -> Option<&TargetNode>`, `children(&self, DomId) -> &[DomId]`, `len(&self) -> usize`,
  `SplitBuffer { tree: TargetTree, passthrough: Vec<DriverDomCommand> }`,
  `split_buffer(Vec<DriverDomCommand>) -> SplitBuffer`.

- [ ] **Step 1: Napisz failujące testy indeksu**

Utwórz `crates/vertigo/src/driver_module/hydration/target_tree.rs` z blokiem testów:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u64) -> DomId {
        DomId::from_u64(value)
    }

    fn element(value: u64, name: &'static str) -> DriverDomCommand {
        DriverDomCommand::CreateNode {
            id: id(value),
            name: name.into(),
        }
    }

    fn insert(parent: u64, child: u64) -> DriverDomCommand {
        DriverDomCommand::InsertBefore {
            parent: id(parent),
            child: id(child),
            ref_id: None,
        }
    }

    #[test]
    fn builds_a_tree_from_creates_and_inserts() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            insert(3, 4),
            element(5, "span"),
            insert(4, 5),
        ]);

        assert_eq!(split.tree.children(id(3)), &[id(4)]);
        assert_eq!(split.tree.children(id(4)), &[id(5)]);
        assert!(split.passthrough.is_empty());
    }

    /// `InsertBefore` z `ref_id` wstawia przed wskazanym rodzeństwem, nie na koniec.
    #[test]
    fn insert_before_respects_the_reference() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            insert(3, 4),
            element(5, "span"),
            DriverDomCommand::InsertBefore {
                parent: id(3),
                child: id(5),
                ref_id: Some(id(4)),
            },
        ]);

        assert_eq!(split.tree.children(id(3)), &[id(5), id(4)]);
    }

    /// Bez odczepienia od poprzedniego rodzica przeniesiony węzeł widniałby w dwóch
    /// miejscach i matcher potknąłby się na kopii, której w DOM nie ma. Serwer robi to samo
    /// przy replayu (`AllElements::insert_before` woła `remove_from_parent`).
    #[test]
    fn a_moved_node_leaves_its_previous_parent() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            element(5, "div"),
            element(6, "span"),
            insert(3, 4),
            insert(3, 5),
            insert(4, 6),
            insert(5, 6),
        ]);

        assert_eq!(split.tree.children(id(4)), &[] as &[DomId]);
        assert_eq!(split.tree.children(id(5)), &[id(6)]);
    }

    /// `CreateText` może nieść nieaktualną wartość: `Computed` odczytany przy otwartej
    /// transakcji zwraca wartość z cache, więc `DomText::patched` potrafi zapiec starą treść
    /// i poprawić ją natychmiast po. Liczy się to, czym węzeł kończy paczkę.
    #[test]
    fn update_text_wins_over_create_text() {
        let split = split_buffer(vec![
            DriverDomCommand::CreateText {
                id: id(4),
                value: "stale".to_string(),
            },
            DriverDomCommand::UpdateText {
                id: id(4),
                value: "fresh".to_string(),
            },
        ]);

        let Some(node) = split.tree.get(id(4)) else {
            panic!("the text node should be in the tree");
        };

        assert_eq!(node.kind, TargetKind::Text {
            value: "fresh".to_string()
        });
    }

    #[test]
    fn attributes_are_accumulated_and_removed() {
        let split = split_buffer(vec![
            element(4, "div"),
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "class".into(),
                value: "row".to_string(),
            },
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "href".into(),
                value: "/a".to_string(),
            },
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "class".into(),
                value: "col".to_string(),
            },
            DriverDomCommand::RemoveAttr {
                id: id(4),
                name: "href".into(),
            },
        ]);

        let Some(node) = split.tree.get(id(4)) else {
            panic!("the element should be in the tree");
        };

        assert_eq!(node.attrs.len(), 1);
        assert_eq!(node.attrs.get(&StaticString::from("class")), Some(&"col".to_string()));
    }

    /// Węzeł utworzony i usunięty w obrębie jednego mountu nie istnieje.
    #[test]
    fn a_node_created_and_removed_is_gone() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            insert(3, 4),
            DriverDomCommand::RemoveNode { id: id(4) },
        ]);

        assert!(split.tree.get(id(4)).is_none());
        assert_eq!(split.tree.children(id(3)), &[] as &[DomId]);
    }

    /// Css i callbacki nie mają związku z tożsamością węzłów, więc przechodzą bez zmian.
    /// Callbacki są kluczowane przez `DomId`, który hydracja zachowuje.
    #[test]
    fn css_and_callbacks_pass_through_in_order() {
        let split = split_buffer(vec![
            element(4, "div"),
            DriverDomCommand::InsertCss {
                selector: Some(".a".to_string()),
                value: "color:red".to_string(),
            },
            DriverDomCommand::CallbackAdd {
                id: id(4),
                event_name: "click".to_string(),
                callback_id: crate::dev::CallbackId::from_u64(7),
            },
        ]);

        assert_eq!(split.passthrough.len(), 2);
        assert!(matches!(
            split.passthrough[0],
            DriverDomCommand::InsertCss { .. }
        ));
        assert!(matches!(
            split.passthrough[1],
            DriverDomCommand::CallbackAdd { .. }
        ));
    }
}
```

- [ ] **Step 2: Uruchom testy, żeby sprawdzić, że nie kompilują się**

Run: `cargo test -p vertigo --all-features target_tree`
Expected: FAIL — `cannot find function split_buffer in this scope`

- [ ] **Step 3: Dopisz implementację nad blokiem testów**

```rust
use std::collections::{BTreeMap, HashMap};

use crate::{
    dev::command::DriverDomCommand, dom::dom_id::DomId, driver_module::StaticString,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TargetKind {
    Element { name: StaticString },
    Text { value: String },
    Comment { value: String },
}

#[derive(Debug, Clone)]
pub struct TargetNode {
    pub kind: TargetKind,
    pub attrs: BTreeMap<StaticString, String>,
    pub children: Vec<DomId>,
}

/// Drzewo, które aplikacja właśnie zbudowała, odtworzone z jej własnego strumienia komend.
///
/// Rust nie zna zawartości swojego drzewa `DomNode`: `DomElement` nie pamięta atrybutów
/// (`add_attr` zapisuje je wprost do drivera przez domknięcie), a `DomText` nie pamięta
/// treści. Opisem drzewa jest więc strumień komend, i to z niego budowany jest ten indeks.
///
/// Żyje tylko przez czas `DriverDom::flush_hydration` i jest po nim porzucany.
#[derive(Debug, Default)]
pub struct TargetTree {
    nodes: HashMap<DomId, TargetNode>,
    /// Rodzic każdego wstawionego węzła. Trzymany osobno, żeby odczepienie było kosztem
    /// jednego rodzeństwa, a nie przejściem po wszystkich węzłach - odpowiednik w js
    /// przeszukuje przy każdym `InsertBefore` całą mapę, co dla dużej strony jest
    /// kwadratowe.
    parent: HashMap<DomId, DomId>,
}

pub struct SplitBuffer {
    pub tree: TargetTree,
    /// Komendy niezwiązane z tożsamością węzłów, wysyłane bez zmian.
    pub passthrough: Vec<DriverDomCommand>,
}

/// Rozdziela bufor mountu na indeks drzewa i komendy przechodzące bez zmian.
pub fn split_buffer(commands: Vec<DriverDomCommand>) -> SplitBuffer {
    let mut tree = TargetTree::default();
    let mut passthrough = Vec::new();

    for command in commands {
        match command {
            DriverDomCommand::CreateNode { id, name } => {
                tree.insert(id, TargetKind::Element { name });
            }
            DriverDomCommand::CreateText { id, value } => {
                tree.insert(id, TargetKind::Text { value });
            }
            DriverDomCommand::CreateComment { id, value } => {
                tree.insert(id, TargetKind::Comment { value });
            }
            DriverDomCommand::UpdateText { id, value } => {
                if let Some(node) = tree.nodes.get_mut(&id) {
                    node.kind = TargetKind::Text { value };
                }
            }
            DriverDomCommand::SetAttr { id, name, value } => {
                if let Some(node) = tree.nodes.get_mut(&id) {
                    node.attrs.insert(name, value);
                }
            }
            DriverDomCommand::RemoveAttr { id, name } => {
                if let Some(node) = tree.nodes.get_mut(&id) {
                    node.attrs.remove(&name);
                }
            }
            DriverDomCommand::InsertBefore {
                parent,
                child,
                ref_id,
            } => {
                tree.unlink(child);
                tree.link(parent, child, ref_id);
            }
            DriverDomCommand::RemoveNode { id }
            | DriverDomCommand::RemoveText { id }
            | DriverDomCommand::RemoveComment { id } => {
                tree.unlink(id);
                tree.nodes.remove(&id);
            }
            other => passthrough.push(other),
        }
    }

    SplitBuffer { tree, passthrough }
}

impl TargetTree {
    fn insert(&mut self, id: DomId, kind: TargetKind) {
        self.nodes.insert(
            id,
            TargetNode {
                kind,
                attrs: BTreeMap::new(),
                children: Vec::new(),
            },
        );
    }

    fn unlink(&mut self, child: DomId) {
        let Some(parent) = self.parent.remove(&child) else {
            return;
        };

        if let Some(node) = self.nodes.get_mut(&parent)
            && let Some(at) = node.children.iter().position(|item| *item == child)
        {
            node.children.remove(at);
        }
    }

    fn link(&mut self, parent: DomId, child: DomId, ref_id: Option<DomId>) {
        let Some(node) = self.nodes.get_mut(&parent) else {
            return;
        };

        match ref_id.and_then(|ref_id| node.children.iter().position(|item| *item == ref_id)) {
            Some(at) => node.children.insert(at, child),
            None => node.children.push(child),
        }

        self.parent.insert(child, parent);
    }

    pub fn get(&self, id: DomId) -> Option<&TargetNode> {
        self.nodes.get(&id)
    }

    pub fn children(&self, id: DomId) -> &[DomId] {
        match self.nodes.get(&id) {
            Some(node) => node.children.as_slice(),
            None => &[],
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
```

W `crates/vertigo/src/driver_module/hydration/mod.rs` dopisz:

```rust
mod target_tree;

pub use target_tree::{SplitBuffer, TargetKind, TargetNode, TargetTree, split_buffer};
```

- [ ] **Step 4: Uruchom testy**

Run: `cargo test -p vertigo --all-features target_tree`
Expected: PASS (7 testów)

- [ ] **Step 5: Commit**

```bash
git add crates/vertigo/src/driver_module/hydration
git commit -m "feat(hydration): build the target tree index from the command buffer"
```

---

### Task 4: Dopasowanie elementów, atrybutów i resztek

**Files:**
- Create: `crates/vertigo/src/driver_module/hydration/report.rs`
- Create: `crates/vertigo/src/driver_module/hydration/matcher.rs`
- Modify: `crates/vertigo/src/driver_module/hydration/mod.rs`

**Interfaces:**
- Consumes: `SplitBuffer`, `TargetTree`, `TargetKind`, `split_buffer` (Task 3); `DomSnapshot`, `SnapshotNode`, `SnapshotAttr` (Task 1); `DriverDomCommand::{NodeAdopt, SnapshotRemove}` (Task 2).
- Produces: `HydrationReport { root_found: bool, matched: u64, hydratable: u64, skipped: u64, total: u64 }` z `publish(&self)`,
  `Reconciled { commands: Vec<DriverDomCommand>, report: HydrationReport }`,
  `reconcile(SplitBuffer, &DomSnapshot) -> Reconciled`,
  `discard(SplitBuffer, &DomSnapshot) -> Vec<DriverDomCommand>`.

- [ ] **Step 1: Napisz raport**

Utwórz `crates/vertigo/src/driver_module/hydration/report.rs`:

```rust
use vertigo_macro::AutoJsJson;

use crate::{JsJsonSerialize, driver_module::api::DomAccess};

/// Co hydracja zrobiła z paczką, którą dostała.
///
/// Parkowany na `window.__vertigo_hydration`, bo tak czyta go test przeglądarkowy
/// (`tests/demo/ssr.rs`): wasm startuje asynchronicznie, więc nie ma momentu, w którym test
/// mógłby podstawić atrapę na `console.log` i być pewnym, że złapie wypisaną linię.
///
/// Nazwy pól są celowo w camelCase - to kontrakt z tym testem, starszy niż ta wersja
/// hydracji.
#[derive(AutoJsJson, Debug, Clone, Default)]
pub struct HydrationReport {
    /// Fałsz, gdy snapshot nie miał `<body>` - hydracja nie miała od czego zacząć.
    #[js_json(rename = "rootFound")]
    pub root_found: bool,
    /// Liczba wyemitowanych adopcji.
    pub matched: u64,
    /// Węzły, które hydracja miała szansę dopasować, czyli elementy i teksty pod
    /// `<head>`/`<body>`. Bez tego, co dopasować się nie da - patrz `skipped`.
    pub hydratable: u64,
    /// Węzły bez odpowiednika w wyjściu serwera: markery komentarzy, które serwer wycina, i
    /// teksty poza pierwszym w scalonym ciągu, które serwer zlepił w jeden przebieg. Nie
    /// liczą się przeciw wynikowi, bo dopasowanie ich jest niemożliwe, nie nieudane.
    pub skipped: u64,
    /// Każdy identyfikator, o którym mówiła paczka.
    pub total: u64,
}

impl HydrationReport {
    pub fn publish(&self) {
        DomAccess::default()
            .root("window")
            .set("__vertigo_hydration", self.to_json())
            .exec();

        let percent = match self.hydratable {
            0 => 100.0,
            hydratable => self.matched as f64 * 100.0 / hydratable as f64,
        };

        let summary = format!(
            "Hydration complete: {}/{} matched ({percent:.2}%), {} skipped, {} nodes in batch.",
            self.matched, self.hydratable, self.skipped, self.total
        );

        if self.matched < self.hydratable {
            log::warn!("{summary}");
        } else {
            log::info!("{summary}");
        }
    }
}
```

- [ ] **Step 2: Napisz failujące testy dopasowania elementów**

Utwórz `crates/vertigo/src/driver_module/hydration/matcher.rs` z blokiem testów. Te pomocnicze funkcje będą używane również w Tasku 5, więc nazwij je dokładnie tak:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver_module::hydration::{SnapshotAttr, split_buffer};

    fn id(value: u64) -> DomId {
        DomId::from_u64(value)
    }

    fn element(value: u64, name: &'static str) -> DriverDomCommand {
        DriverDomCommand::CreateNode {
            id: id(value),
            name: name.into(),
        }
    }

    fn text(value: u64, content: &str) -> DriverDomCommand {
        DriverDomCommand::CreateText {
            id: id(value),
            value: content.to_string(),
        }
    }

    fn insert(parent: u64, child: u64) -> DriverDomCommand {
        DriverDomCommand::InsertBefore {
            parent: id(parent),
            child: id(child),
            ref_id: None,
        }
    }

    fn attr(id_value: u64, name: &'static str, value: &str) -> DriverDomCommand {
        DriverDomCommand::SetAttr {
            id: id(id_value),
            name: name.into(),
            value: value.to_string(),
        }
    }

    fn snap_element(name: &str, children: Vec<u32>) -> SnapshotNode {
        SnapshotNode::Element {
            name: name.to_string(),
            attrs: vec![],
            children,
        }
    }

    fn snap_element_attrs(name: &str, attrs: Vec<(&str, &str)>, children: Vec<u32>) -> SnapshotNode {
        SnapshotNode::Element {
            name: name.to_string(),
            attrs: attrs
                .into_iter()
                .map(|(name, value)| SnapshotAttr {
                    name: name.to_string(),
                    value: value.to_string(),
                })
                .collect(),
            children,
        }
    }

    fn snap_text(value: &str) -> SnapshotNode {
        SnapshotNode::Text {
            value: value.to_string(),
        }
    }

    /// Snapshot o kształcie `<html><head/><body>{body_children}</body></html>`, gdzie węzły
    /// ciała zaczynają się od indeksu 3.
    fn document(body_children: Vec<SnapshotNode>) -> DomSnapshot {
        let count = body_children.len() as u32;
        let mut nodes = vec![
            snap_element("html", vec![1, 2]),
            snap_element("head", vec![]),
            snap_element("body", (0..count).map(|offset| offset + 3).collect()),
        ];
        nodes.extend(body_children);

        DomSnapshot {
            nodes,
            head: Some(1),
            body: Some(2),
        }
    }

    /// Minimalny bufor mountu: `<html>`, `<head>`, `<body>` i to, co poniżej.
    fn mount_buffer(below_body: Vec<DriverDomCommand>) -> Vec<DriverDomCommand> {
        let mut commands = vec![
            element(1, "html"),
            element(2, "head"),
            insert(1, 2),
            element(3, "body"),
            insert(1, 3),
        ];
        commands.extend(below_body);
        commands
    }

    fn run(below_body: Vec<DriverDomCommand>, snapshot: &DomSnapshot) -> Reconciled {
        reconcile(split_buffer(mount_buffer(below_body)), snapshot)
    }

    fn adopted(commands: &[DriverDomCommand]) -> Vec<(u64, u32)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::NodeAdopt { id, snapshot } => Some((id.to_u64(), *snapshot)),
                _ => None,
            })
            .collect()
    }

    fn removed(commands: &[DriverDomCommand]) -> Vec<u32> {
        commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::SnapshotRemove { snapshot } => Some(*snapshot),
                _ => None,
            })
            .collect()
    }

    fn created(commands: &[DriverDomCommand]) -> Vec<u64> {
        commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::CreateNode { id, .. }
                | DriverDomCommand::CreateText { id, .. }
                | DriverDomCommand::CreateComment { id, .. } => Some(id.to_u64()),
                _ => None,
            })
            .collect()
    }

    /// Węzeł, który serwer już wyrenderował, jest przejmowany, nie tworzony na nowo.
    #[test]
    fn a_matching_element_is_adopted() {
        let snapshot = document(vec![snap_element("div", vec![])]);

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);
        assert!(created(&result.commands).is_empty());
        assert_eq!(result.report.matched, 1);
        assert_eq!(result.report.hydratable, 1);
        assert!(result.report.root_found);
    }

    /// Węzeł adoptowany, który już stoi na swoim miejscu, nie generuje `InsertBefore`.
    /// Stąd bierze się skurczenie startowej paczki.
    #[test]
    fn an_adopted_node_in_place_needs_no_insert() {
        let snapshot = document(vec![snap_element("div", vec![]), snap_element("span", vec![])]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                element(5, "span"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
        assert!(
            !result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::InsertBefore { .. })),
            "nothing moved, so nothing should be inserted: {:?}",
            result.commands
        );
    }

    /// Dopasowanie schodzi w głąb.
    #[test]
    fn matching_recurses_into_children() {
        let snapshot = document(vec![snap_element("div", vec![4]), snap_text("hello")]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                text(5, "hello"),
                insert(4, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
    }

    /// Atrybuty uzgadniane są w obie strony. Serwer mógł wyrenderować atrybut, którego
    /// drzewo klienckie nie ma - renderuje z tego samego strumienia, ale komponent może pod
    /// `is_browser()` narysować coś innego, i wtedy zostawiony `href` przeżyłby na węźle,
    /// który należy już do przeglądarki.
    #[test]
    fn attributes_are_reconciled_in_both_directions() {
        let snapshot = document(vec![snap_element_attrs(
            "a",
            vec![("href", "/server"), ("title", "stays")],
            vec![],
        )]);

        let result = run(
            vec![
                element(4, "a"),
                attr(4, "title", "stays"),
                attr(4, "class", "new"),
                insert(3, 4),
            ],
            &snapshot,
        );

        let sets: Vec<(&str, &str)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::SetAttr { name, value, .. } => {
                    Some((name.as_str(), value.as_str()))
                }
                _ => None,
            })
            .collect();

        let removes: Vec<&str> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::RemoveAttr { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(sets, vec![("class", "new")], "only the difference is sent");
        assert_eq!(removes, vec!["href"], "the server's leftover goes away");
    }

    /// Węzeł bez odpowiednika powstaje od nowa i zostaje wstawiony przed najbliższym
    /// adoptowanym rodzeństwem.
    #[test]
    fn an_unmatched_target_is_created_before_the_next_adopted_sibling() {
        let snapshot = document(vec![snap_element("span", vec![])]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                element(5, "span"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(5, 3)]);
        assert_eq!(created(&result.commands), vec![4]);

        let inserts: Vec<(u64, u64, Option<u64>)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } => Some((parent.to_u64(), child.to_u64(), ref_id.map(|id| id.to_u64()))),
                _ => None,
            })
            .collect();

        assert_eq!(inserts, vec![(3, 4, Some(5))]);
    }

    /// Brak dopasowania nie jest dowodem, że zawartość snapshotu jest śmieciem: tworzymy
    /// węzeł, ale nie przesuwamy kursora ani nic nie usuwamy *w tym kroku*. Snapshotowy
    /// `<span>` ginie dopiero na końcu, jako resztka, której nikt nie przejął - a nie jako
    /// węzeł przeskoczony w drodze do trafienia.
    #[test]
    fn a_missing_match_leaves_the_cursor_where_it_was() {
        let snapshot = document(vec![snap_element("span", vec![])]);

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert!(created(&result.commands).contains(&4));
        assert_eq!(
            removed(&result.commands),
            vec![3],
            "the span is a leftover only because no target child ever claimed it"
        );
    }

    /// Węzły przeskoczone po drodze do trafienia są usuwane, po jednej komendzie na korzeń
    /// odrzuconego poddrzewa.
    ///
    /// Snapshot budowany tu wprost, bo `document` wiesza wszystkie przekazane węzły pod
    /// `<body>`, a tutaj potrzebne jest zagnieżdżenie.
    #[test]
    fn skipped_nodes_are_removed_once_per_subtree() {
        let snapshot = DomSnapshot {
            nodes: vec![
                snap_element("html", vec![1, 2]),
                snap_element("head", vec![]),
                snap_element("body", vec![3, 5]),
                snap_element("p", vec![4]),
                snap_text("deep inside the p"),
                snap_element("div", vec![]),
            ],
            head: Some(1),
            body: Some(2),
        };

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 5)]);
        assert_eq!(
            removed(&result.commands),
            vec![3],
            "removing the <p> takes its subtree with it - the text inside needs no command \
             of its own"
        );
    }

    /// Snapshot bez `<body>` znaczy, że hydracja nie ma od czego zacząć przejścia. Wracamy
    /// wtedy do strumienia bez hydracji: całe drzewo powstaje od nowa, nic nie jest
    /// adoptowane, a `root_found` mówi o tym wprost.
    #[test]
    fn a_snapshot_without_a_body_falls_back_to_building_everything() {
        let snapshot = DomSnapshot {
            nodes: vec![snap_element("html", vec![])],
            head: None,
            body: None,
        };

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert!(!result.report.root_found);
        assert!(adopted(&result.commands).is_empty());
        assert!(removed(&result.commands).is_empty());
        assert!(
            created(&result.commands).contains(&4),
            "the app still has to be built: {:?}",
            result.commands
        );
    }

    /// Atrybuty korzeni uzgadniane są mimo że same nie są adoptowane - `MapNodes` rozwiązuje
    /// id 1, 2 i 3 dynamicznie.
    #[test]
    fn the_document_roots_are_not_adopted_but_their_attributes_are() {
        let mut snapshot = document(vec![]);
        snapshot.nodes[0] = snap_element_attrs("html", vec![("lang", "en")], vec![1, 2]);

        let mut below = vec![attr(1, "lang", "pl")];
        below.push(attr(3, "class", "page"));

        let result = run(below, &snapshot);

        assert!(adopted(&result.commands).is_empty());

        let sets: Vec<(u64, &str, &str)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::SetAttr { id, name, value } => {
                    Some((id.to_u64(), name.as_str(), value.as_str()))
                }
                _ => None,
            })
            .collect();

        assert!(sets.contains(&(1, "lang", "pl")));
        assert!(sets.contains(&(3, "class", "page")));
    }

    /// Grupa przechodząca trafia do wyniku nietknięta.
    #[test]
    fn passthrough_commands_are_kept() {
        let snapshot = document(vec![snap_element("div", vec![])]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                DriverDomCommand::CallbackAdd {
                    id: id(4),
                    event_name: "click".to_string(),
                    callback_id: crate::dev::CallbackId::from_u64(7),
                },
                DriverDomCommand::InsertCss {
                    selector: None,
                    value: "a{}".to_string(),
                },
            ],
            &snapshot,
        );

        assert!(
            result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::CallbackAdd { .. }))
        );
        assert!(
            result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::InsertCss { .. }))
        );
    }

    /// Przy wyłączonej hydracji nie adoptujemy niczego, usuwamy zawartość serwera i
    /// wysyłamy bufor bez zmian. Dziś odpowiada temu `removeInitNodes` w js.
    #[test]
    fn discard_removes_the_server_markup_and_keeps_the_buffer() {
        let snapshot = document(vec![snap_element("div", vec![]), snap_text("hello")]);

        let commands = discard(split_buffer(mount_buffer(vec![])), &snapshot);

        assert_eq!(removed(&commands), vec![3, 4]);
        assert!(adopted(&commands).is_empty());
        assert!(created(&commands).contains(&3), "the buffer is untouched");
    }
}
```

- [ ] **Step 3: Uruchom testy, żeby sprawdzić, że nie kompilują się**

Run: `cargo test -p vertigo --all-features matcher`
Expected: FAIL — `cannot find function reconcile in this scope`

- [ ] **Step 4: Dopisz implementację nad blokiem testów**

Uwaga na pożyczki: `tree` i `snapshot` są w `Matcher` referencjami o czasie życia `'a`. Zanim sięgniesz po dane drzewa i zaczniesz pisać do `self.out`, przepisz referencję do zmiennej lokalnej (`let tree = self.tree;`) — inaczej pożyczka pola przez metodę na `self` zderzy się z mutowalną pożyczką `self.out`.

```rust
use std::collections::HashSet;

use super::{
    report::HydrationReport,
    snapshot::{DomSnapshot, SnapshotNode},
    target_tree::{SplitBuffer, TargetKind, TargetNode, TargetTree},
};
use crate::{dev::command::DriverDomCommand, dom::dom_id::DomId, driver_module::StaticString};

const HTML_ID: u64 = 1;
const HEAD_ID: u64 = 2;
const BODY_ID: u64 = 3;

pub struct Reconciled {
    pub commands: Vec<DriverDomCommand>,
    pub report: HydrationReport,
}

/// Uzgadnia drzewo, które aplikacja zbudowała, ze stanem DOM przeglądarki.
///
/// Wynik to zredukowany strumień: adopcje istniejących węzłów, tworzenie tylko tego, czego
/// serwer nie wyrenderował, poprawki wyłącznie tam, gdzie coś się różni, i usunięcie resztek.
pub fn reconcile(split: SplitBuffer, snapshot: &DomSnapshot) -> Reconciled {
    let SplitBuffer { tree, passthrough } = split;

    let report = HydrationReport {
        total: tree.len() as u64,
        ..HydrationReport::default()
    };

    // Bez `<body>` nie ma od czego zacząć przejścia. Zamiast adoptować cokolwiek na oślep,
    // wracamy do strumienia bez hydracji - ta sama decyzja, którą podejmuje dziś `hydrate`,
    // gdy w paczce nie ma id 3.
    let Some(body) = snapshot.body else {
        let mut commands = rebuild_verbatim(&tree);
        commands.extend(passthrough);

        return Reconciled { commands, report };
    };

    let mut matcher = Matcher {
        tree: &tree,
        snapshot,
        out: Vec::new(),
        report: HydrationReport {
            root_found: true,
            ..report
        },
    };

    if !snapshot.nodes.is_empty() {
        matcher.reconcile_attrs(DomId::from_u64(HTML_ID), 0);
    }

    matcher.reconcile_root(DomId::from_u64(BODY_ID), body);

    if let Some(head) = snapshot.head {
        matcher.reconcile_root(DomId::from_u64(HEAD_ID), head);
    }

    let mut commands = matcher.out;
    commands.extend(passthrough);

    Reconciled {
        commands,
        report: matcher.report,
    }
}

/// Wariant dla `--disable-hydration`: nic nie jest adoptowane, zawartość serwera jest
/// usuwana, a bufor mountu leci bez zmian.
pub fn discard(split: SplitBuffer, snapshot: &DomSnapshot) -> Vec<DriverDomCommand> {
    let SplitBuffer { tree, passthrough } = split;

    let mut commands = Vec::new();

    for root in [snapshot.body, snapshot.head].into_iter().flatten() {
        for child in snapshot.children(root) {
            commands.push(DriverDomCommand::SnapshotRemove { snapshot: *child });
        }
    }

    commands.extend(rebuild_verbatim(&tree));
    commands.extend(passthrough);
    commands
}

/// Bufor mountu w postaci, w jakiej wyszedłby bez hydracji.
///
/// `split_buffer` pochłonęło oryginalne komendy strukturalne, więc obie ścieżki, które mają
/// zbudować drzewo od zera - brak `<body>` w snapshocie i `--disable-hydration` - odtwarzają
/// je z indeksu. Kolejność jest dokumentowa: rodzic przed dziećmi.
fn rebuild_verbatim(tree: &TargetTree) -> Vec<DriverDomCommand> {
    let mut out = Vec::new();
    let mut visited: HashSet<DomId> = HashSet::new();

    for id in [HTML_ID, HEAD_ID, BODY_ID] {
        let id = DomId::from_u64(id);
        if let Some(node) = tree.get(id) {
            out.extend(create_commands(id, node));
        }
    }

    // Trzy wejścia, a nie jedno, bo `<html>` może w drzewie nie być - aplikacja montowana
    // przez `start_app` bez własnego `<html>` dostaje same `<head>` i `<body>`. Zbiór
    // odwiedzonych pilnuje, żeby `<head>` i `<body>` osiągnięte z `<html>` nie zostały
    // obeszły po raz drugi.
    for root in [HTML_ID, HEAD_ID, BODY_ID] {
        rebuild_children(tree, DomId::from_u64(root), &mut visited, &mut out);
    }

    out
}

fn rebuild_children(
    tree: &TargetTree,
    parent: DomId,
    visited: &mut HashSet<DomId>,
    out: &mut Vec<DriverDomCommand>,
) {
    if !visited.insert(parent) {
        return;
    }

    for child in tree.children(parent) {
        let child = *child;

        // Korzenie dokumentu już są - `MapNodes` rozwiązuje ich id dynamicznie.
        if !matches!(child.to_u64(), HTML_ID | HEAD_ID | BODY_ID)
            && let Some(node) = tree.get(child)
        {
            out.extend(create_commands(child, node));
        }

        out.push(DriverDomCommand::InsertBefore {
            parent,
            child,
            ref_id: None,
        });

        rebuild_children(tree, child, visited, out);
    }
}

fn create_commands(id: DomId, node: &TargetNode) -> Vec<DriverDomCommand> {
    let mut out = Vec::new();

    match &node.kind {
        TargetKind::Element { name } => {
            out.push(DriverDomCommand::CreateNode {
                id,
                name: name.clone(),
            });
        }
        TargetKind::Text { value } => out.push(DriverDomCommand::CreateText {
            id,
            value: value.clone(),
        }),
        TargetKind::Comment { value } => out.push(DriverDomCommand::CreateComment {
            id,
            value: value.clone(),
        }),
    }

    for (name, value) in &node.attrs {
        out.push(DriverDomCommand::SetAttr {
            id,
            name: name.clone(),
            value: value.clone(),
        });
    }

    out
}

/// Co zapada o jednym dziecku docelowym w pierwszym przebiegu.
///
/// Dwa przebiegi, a nie jeden, bo `InsertBefore` dla tworzonego węzła potrzebuje
/// identyfikatora **następnego** adoptowanego rodzeństwa - a to jest wiedza o przyszłości,
/// gdyby emitować w trakcie decydowania.
enum ChildPlan {
    Adopt { child: DomId, snapshot: u32 },
    AdoptText { child: DomId, snapshot: u32, patch: bool },
    Create { child: DomId },
}

impl ChildPlan {
    fn child(&self) -> DomId {
        match self {
            Self::Adopt { child, .. } | Self::AdoptText { child, .. } | Self::Create { child } => {
                *child
            }
        }
    }

    fn is_adopted(&self) -> bool {
        !matches!(self, Self::Create { .. })
    }
}

struct Matcher<'a> {
    tree: &'a TargetTree,
    snapshot: &'a DomSnapshot,
    out: Vec<DriverDomCommand>,
    report: HydrationReport,
}

impl<'a> Matcher<'a> {
    fn reconcile_root(&mut self, root: DomId, snapshot_index: u32) {
        self.reconcile_attrs(root, snapshot_index);
        self.reconcile_children(root, snapshot_index);
    }

    fn reconcile_attrs(&mut self, id: DomId, snapshot_index: u32) {
        let tree = self.tree;
        let snapshot = self.snapshot;

        let Some(node) = tree.get(id) else {
            return;
        };

        let existing = snapshot.attrs(snapshot_index);

        for attr in existing {
            let wanted = node.attrs.keys().any(|key| key.as_str() == attr.name);
            if !wanted {
                self.out.push(DriverDomCommand::RemoveAttr {
                    id,
                    name: StaticString::from(attr.name.clone()),
                });
            }
        }

        for (name, value) in &node.attrs {
            let same = existing
                .iter()
                .any(|attr| attr.name == name.as_str() && &attr.value == value);

            if !same {
                self.out.push(DriverDomCommand::SetAttr {
                    id,
                    name: name.clone(),
                    value: value.clone(),
                });
            }
        }
    }

    fn reconcile_children(&mut self, parent: DomId, parent_snapshot: u32) {
        let (plans, removals) = self.plan_children(parent, parent_snapshot);
        self.emit_children(parent, &plans);

        for snapshot in removals {
            self.out
                .push(DriverDomCommand::SnapshotRemove { snapshot });
        }
    }

    /// Pierwszy przebieg: kto adoptuje który węzeł snapshotu, a kto powstaje od nowa.
    fn plan_children(&mut self, parent: DomId, parent_snapshot: u32) -> (Vec<ChildPlan>, Vec<u32>) {
        let tree = self.tree;
        let snapshot = self.snapshot;

        let target_children = tree.children(parent);
        let snapshot_children = snapshot.children(parent_snapshot);

        let mut plans = Vec::with_capacity(target_children.len());
        let mut removals = Vec::new();
        let mut cursor = 0usize;
        let mut index = 0usize;

        while index < target_children.len() {
            let child = target_children[index];

            let Some(node) = tree.get(child) else {
                index += 1;
                continue;
            };

            match &node.kind {
                // Markery `render_value`/`render_list` nie mają w html odpowiednika: serwer
                // wycina komentarze (`get_render_child_mode` odrzuca `HtmlNode::Comment`).
                // Nie mają więc czego dopasowywać i nie liczą się przeciw wynikowi.
                TargetKind::Comment { .. } => {
                    self.report.skipped += 1;
                    plans.push(ChildPlan::Create { child });
                    index += 1;
                }
                TargetKind::Element { name } => {
                    self.report.hydratable += 1;

                    match find_element(snapshot, snapshot_children, cursor, name.as_str()) {
                        Some(found) => {
                            removals.extend_from_slice(&snapshot_children[cursor..found]);
                            let snapshot_index = snapshot_children[found];
                            plans.push(ChildPlan::Adopt {
                                child,
                                snapshot: snapshot_index,
                            });
                            cursor = found + 1;
                        }
                        None => plans.push(ChildPlan::Create { child }),
                    }

                    index += 1;
                }
                TargetKind::Text { value } => {
                    let run = text_run_length(tree, target_children, index);
                    let taken = self.plan_text_run(
                        &mut plans,
                        target_children,
                        index,
                        run,
                        snapshot_children,
                        cursor,
                        value,
                    );
                    cursor = taken;
                    index += run;
                }
            }
        }

        removals.extend_from_slice(&snapshot_children[cursor.min(snapshot_children.len())..]);

        (plans, removals)
    }

    /// Drugi przebieg: emisja, w trzech turach po tym samym rodzeństwie.
    ///
    /// Kolejność tur nie jest kosmetyczna. `InsertBefore` dla węzła tworzonego wskazuje jako
    /// punkt odniesienia adoptowane rodzeństwo, a `MapNodes` rozwiąże ten identyfikator
    /// dopiero wtedy, gdy adopcja go zarejestruje - więc wszystkie adopcje rodzeństwa muszą
    /// wyjść przed pierwszym wstawieniem. Zejście w głąb idzie na koniec, bo emituje komendy
    /// dotyczące już innego rodzeństwa i nie ma wpływu na ten poziom.
    fn emit_children(&mut self, parent: DomId, plans: &[ChildPlan]) {
        for plan in plans {
            match plan {
                ChildPlan::Adopt { child, snapshot } => {
                    self.out.push(DriverDomCommand::NodeAdopt {
                        id: *child,
                        snapshot: *snapshot,
                    });
                    self.report.matched += 1;
                    self.reconcile_attrs(*child, *snapshot);
                }
                ChildPlan::AdoptText {
                    child,
                    snapshot,
                    patch,
                } => {
                    self.out.push(DriverDomCommand::NodeAdopt {
                        id: *child,
                        snapshot: *snapshot,
                    });
                    self.report.matched += 1;

                    if *patch {
                        let tree = self.tree;

                        if let Some(node) = tree.get(*child)
                            && let TargetKind::Text { value } = &node.kind
                        {
                            self.out.push(DriverDomCommand::UpdateText {
                                id: *child,
                                value: value.clone(),
                            });
                        }
                    }
                }
                ChildPlan::Create { .. } => {}
            }
        }

        for (at, plan) in plans.iter().enumerate() {
            if let ChildPlan::Create { child } = plan {
                let ref_id = plans
                    .iter()
                    .skip(at + 1)
                    .find(|plan| plan.is_adopted())
                    .map(ChildPlan::child);

                self.create_subtree(*child, parent, ref_id);
            }
        }

        for plan in plans {
            if let ChildPlan::Adopt { child, snapshot } = plan {
                self.reconcile_children(*child, *snapshot);
            }
        }
    }

    fn create_subtree(&mut self, id: DomId, parent: DomId, ref_id: Option<DomId>) {
        let tree = self.tree;

        if let Some(node) = tree.get(id) {
            self.out.extend(create_commands(id, node));
        }

        self.out.push(DriverDomCommand::InsertBefore {
            parent,
            child: id,
            ref_id,
        });

        for child in tree.children(id) {
            self.create_subtree(*child, id, None);
        }
    }
}

/// Nazwa elementu, bez uwzględniania wielkości liter i bez prefiksu `svg:`.
///
/// Js wysyła `tagName` zmniejszone do małych liter, a porównanie bez wielkości liter jest
/// poprawne dla html i svg naraz - co oszczędza przeniesienia sześćdziesięcioelementowego
/// zbioru `SVG_TAGS` do wasma. `tags.ts` zostaje w js, bo `createElement` i tak potrzebuje
/// go do wyboru przestrzeni nazw.
fn tag_matches(target: &str, candidate: &str) -> bool {
    let local = target.strip_prefix("svg:").unwrap_or(target);
    local.eq_ignore_ascii_case(candidate)
}

fn find_element(
    snapshot: &DomSnapshot,
    children: &[u32],
    from: usize,
    name: &str,
) -> Option<usize> {
    for (at, index) in children.iter().enumerate().skip(from) {
        if let Some(SnapshotNode::Element {
            name: candidate, ..
        }) = snapshot.node(*index)
            && tag_matches(name, candidate)
        {
            return Some(at);
        }
    }

    None
}

fn text_run_length(tree: &TargetTree, children: &[DomId], from: usize) -> usize {
    let mut length = 0;

    for child in children.iter().skip(from) {
        match tree.get(*child) {
            Some(node) if matches!(node.kind, TargetKind::Text { .. }) => length += 1,
            _ => break,
        }
    }

    length.max(1)
}
```

Metodę `plan_text_run` dopisuje Task 5. Na teraz dodaj tymczasową wersję, która obsługuje wyłącznie ciąg jednoelementowy — testy tekstowe z Taska 4 (`matching_recurses_into_children`) potrzebują tylko tego:

```rust
impl<'a> Matcher<'a> {
    #[allow(clippy::too_many_arguments)]
    fn plan_text_run(
        &mut self,
        plans: &mut Vec<ChildPlan>,
        target_children: &[DomId],
        index: usize,
        _run: usize,
        snapshot_children: &[u32],
        cursor: usize,
        value: &str,
    ) -> usize {
        self.report.hydratable += 1;

        let child = target_children[index];

        match snapshot_children.get(cursor) {
            Some(snapshot_index) if is_text(self.snapshot, *snapshot_index) => {
                let patch = !text_equals(self.snapshot, *snapshot_index, value);
                plans.push(ChildPlan::AdoptText {
                    child,
                    snapshot: *snapshot_index,
                    patch,
                });
                cursor + 1
            }
            _ => {
                plans.push(ChildPlan::Create { child });
                cursor
            }
        }
    }
}

fn is_text(snapshot: &DomSnapshot, index: u32) -> bool {
    matches!(snapshot.node(index), Some(SnapshotNode::Text { .. }))
}

fn text_equals(snapshot: &DomSnapshot, index: u32, value: &str) -> bool {
    matches!(snapshot.node(index), Some(SnapshotNode::Text { value: existing }) if existing == value)
}
```

W `crates/vertigo/src/driver_module/hydration/mod.rs`:

```rust
mod matcher;
mod report;

pub use matcher::{Reconciled, discard, reconcile};
pub use report::HydrationReport;
```

- [ ] **Step 5: Uruchom testy**

Run: `cargo test -p vertigo --all-features matcher`
Expected: PASS (11 testów)

- [ ] **Step 6: Sprawdź clippy — matcher ma rekurencję i dużo wzorców**

Run: `cargo clippy --locked -p vertigo --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/vertigo/src/driver_module/hydration
git commit -m "feat(hydration): match elements and attributes against the snapshot"
```

---

### Task 5: Teksty, białe znaki i wielkość liter w SVG

**Files:**
- Modify: `crates/vertigo/src/driver_module/hydration/matcher.rs`

**Interfaces:**
- Consumes: wszystko z Taska 4 oraz pomocnicze funkcje testowe zdefiniowane w jego bloku `mod tests` (`document`, `run`, `adopted`, `created`, `removed`, `snap_text`, `snap_element`, `element`, `text`, `insert`).
- Produces: pełna wersja `Matcher::plan_text_run` obsługująca scalone ciągi i pomijanie białych znaków.

- [ ] **Step 1: Napisz failujące testy**

Dopisz do bloku `mod tests` w `matcher.rs`:

```rust
    /// Kilka sąsiadujących `DomText` serwer zlepia w jeden przebieg tekstowy
    /// (`last_text_add` w `get_render_child_mode`), a parser robi z tego jeden węzeł.
    ///
    /// Scalony węzeł adoptuje pierwszy tekst i dostaje `UpdateText` obcinający go do własnej
    /// wartości; rodzeństwo powstaje od nowa. Odpowiednik w js liczył pozostałe jako
    /// dopasowane, nie wiążąc ich z niczym - i późniejszy `UpdateText` na drugim z nich
    /// trafiał w id nieobecne w `MapNodes`.
    #[test]
    fn a_merged_text_run_is_split() {
        let snapshot = document(vec![snap_text("firstsecond")]);

        let result = run(
            vec![
                text(4, "first"),
                insert(3, 4),
                text(5, "second"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);
        assert_eq!(created(&result.commands), vec![5]);

        let patches: Vec<(u64, &str)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::UpdateText { id, value } => Some((id.to_u64(), value.as_str())),
                _ => None,
            })
            .collect();

        assert_eq!(
            patches,
            vec![(4, "first")],
            "the adopted node is cut down to its own value"
        );

        assert_eq!(
            result.report.skipped, 1,
            "the second text cannot be matched - the server merged it away - so it must not \
             count against the score"
        );
        assert_eq!(result.report.hydratable, 1);
        assert_eq!(result.report.matched, 1);
    }

    /// Pojedynczy tekst o zgodnej treści nie wymaga poprawki.
    #[test]
    fn an_identical_single_text_is_adopted_without_a_patch() {
        let snapshot = document(vec![snap_text("hello")]);

        let result = run(vec![text(4, "hello"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);
        assert!(
            !result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::UpdateText { .. })),
            "nothing changed, so nothing should be patched"
        );
    }

    #[test]
    fn a_differing_single_text_is_adopted_and_patched() {
        let snapshot = document(vec![snap_text("stale")]);

        let result = run(vec![text(4, "fresh"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);

        let patches: Vec<&str> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::UpdateText { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(patches, vec!["fresh"]);
    }

    /// Produkcyjne renderowanie serwerowe formatuje wyjście (`convert_to_string(true)`), więc
    /// w html są wcięcia, które parser zamienia na węzły tekstowe nieistniejące w drzewie
    /// vertigo. Szukając elementu, wolno je pominąć i usunąć.
    ///
    /// Jest to bezpieczne, bo konteksty się nie nachodzą: tam, gdzie białe znaki są znaczące
    /// (`<pre>`, treść inline), formatowanie przechodzi na `Format::none()` i nic nie
    /// wstrzykuje - a wtedy odpowiedni węzeł tekstowy istnieje też w drzewie docelowym.
    #[test]
    fn formatting_whitespace_is_skipped_when_looking_for_an_element() {
        let snapshot = document(vec![
            snap_text("\n  "),
            snap_element("div", vec![]),
            snap_text("\n"),
        ]);

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 4)]);
        assert_eq!(removed(&result.commands), vec![3, 5]);
        assert!(created(&result.commands).is_empty());
    }

    /// Elementy svg zachowują swoją wielkość liter w `tagName`, html-owe raportują wielkimi.
    /// Js wysyła nazwę małymi literami, a porównanie bez wielkości liter obsługuje oba
    /// światy naraz - dlatego `SVG_TAGS` nie musi trafić do wasma.
    #[test]
    fn svg_casing_matches_without_a_tag_table() {
        let snapshot = document(vec![snap_element("svg", vec![4]), snap_element("lineargradient", vec![])]);

        let result = run(
            vec![
                element(4, "svg"),
                insert(3, 4),
                element(5, "linearGradient"),
                insert(4, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
    }

    /// Nazwa z prefiksem `svg:` dopasowuje się do lokalnej nazwy - js tworzy taki element
    /// przez `createElementNS` po zdjęciu prefiksu, więc parser widzi `<a>`.
    #[test]
    fn an_svg_prefixed_name_matches_its_local_name() {
        let snapshot = document(vec![snap_element("svg", vec![4]), snap_element("a", vec![])]);

        let result = run(
            vec![
                element(4, "svg"),
                insert(3, 4),
                element(5, "svg:a"),
                insert(4, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
    }

    /// Marker komentarza powstaje od nowa i nie konsumuje węzła snapshotu - następny element
    /// docelowy musi trafić w to, co serwer faktycznie wyrenderował.
    #[test]
    fn a_comment_marker_does_not_consume_a_snapshot_node() {
        let snapshot = document(vec![snap_element("div", vec![])]);

        let result = run(
            vec![
                DriverDomCommand::CreateComment {
                    id: id(4),
                    value: "marker".to_string(),
                },
                insert(3, 4),
                element(5, "div"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(5, 3)]);
        assert_eq!(created(&result.commands), vec![4]);
        assert_eq!(result.report.skipped, 1);
        assert_eq!(result.report.hydratable, 1);
        assert!(removed(&result.commands).is_empty());
    }
```

- [ ] **Step 2: Uruchom testy, żeby sprawdzić, które failują**

Run: `cargo test -p vertigo --all-features matcher`
Expected: FAIL — `a_merged_text_run_is_split` i `formatting_whitespace_is_skipped_when_looking_for_an_element`. Pozostałe nowe testy przechodzą już na tymczasowej wersji z Taska 4.

- [ ] **Step 3: Zamień tymczasową `plan_text_run` na pełną**

Usuń wersję dopisaną w Tasku 4 i wstaw:

```rust
impl<'a> Matcher<'a> {
    /// Planuje ciąg `run` następujących po sobie dzieci tekstowych, zaczynając od `index`.
    ///
    /// Rozpoznanie przypadku scalonego odbywa się po stronie drzewa docelowego, nie przez
    /// analizę treści snapshotu: serwer zlepia dokładnie takie ciągi, więc długość ciągu
    /// wystarcza i nie trzeba dopasowywać prefiksów.
    ///
    /// Zwraca nową pozycję kursora w `snapshot_children`.
    #[allow(clippy::too_many_arguments)]
    fn plan_text_run(
        &mut self,
        plans: &mut Vec<ChildPlan>,
        target_children: &[DomId],
        index: usize,
        run: usize,
        snapshot_children: &[u32],
        cursor: usize,
        value: &str,
    ) -> usize {
        // Tylko pierwszy z ciągu ma szansę na dopasowanie. Pozostałe serwer scalił, więc nie
        // mają czego dopasowywać - tak samo jak markery komentarzy, i tak samo nie liczą się
        // przeciw wynikowi.
        self.report.hydratable += 1;
        self.report.skipped += (run - 1) as u64;

        let first = target_children[index];

        let matched = match snapshot_children.get(cursor) {
            Some(snapshot_index) if is_text(self.snapshot, *snapshot_index) => {
                let patch = run > 1 || !text_equals(self.snapshot, *snapshot_index, value);

                plans.push(ChildPlan::AdoptText {
                    child: first,
                    snapshot: *snapshot_index,
                    patch,
                });

                cursor + 1
            }
            _ => {
                plans.push(ChildPlan::Create { child: first });
                cursor
            }
        };

        for child in target_children.iter().skip(index + 1).take(run - 1) {
            plans.push(ChildPlan::Create { child: *child });
        }

        matched
    }
}
```

- [ ] **Step 4: Dodaj pomijanie białych znaków przy szukaniu elementu**

Dopisz predykat obok `text_run_length` (w Tasku 4 go nie było, bo bez użycia clippy zgłosiłoby martwy kod):

```rust
fn is_whitespace(snapshot: &DomSnapshot, index: u32) -> bool {
    match snapshot.node(index) {
        Some(SnapshotNode::Text { value }) => value.trim().is_empty(),
        _ => false,
    }
}
```

W `plan_children`, w gałęzi `TargetKind::Element`, zamień wyszukiwanie tak, żeby najpierw przeskoczyć białe znaki z formatowania. Podmień gałąź `Element` na:

```rust
                TargetKind::Element { name } => {
                    self.report.hydratable += 1;

                    let mut from = cursor;
                    while snapshot_children
                        .get(from)
                        .is_some_and(|index| is_whitespace(snapshot, *index))
                    {
                        from += 1;
                    }

                    match find_element(snapshot, snapshot_children, from, name.as_str()) {
                        Some(found) => {
                            removals.extend_from_slice(&snapshot_children[cursor..found]);
                            let snapshot_index = snapshot_children[found];
                            plans.push(ChildPlan::Adopt {
                                child,
                                snapshot: snapshot_index,
                            });
                            cursor = found + 1;
                        }
                        None => plans.push(ChildPlan::Create { child }),
                    }

                    index += 1;
                }
```

Zauważ, że `removals` liczone jest od `cursor`, nie od `from` — przeskoczone białe znaki mają zostać usunięte, a nie zapomniane.

- [ ] **Step 5: Uruchom testy**

Run: `cargo test -p vertigo --all-features matcher`
Expected: PASS (18 testów)

- [ ] **Step 6: Uruchom całość i clippy**

Run: `cargo test --all-features && cargo clippy --locked -p vertigo --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/vertigo/src/driver_module/hydration/matcher.rs
git commit -m "feat(hydration): split merged text runs, skip formatting whitespace"
```

---

### Task 6: Wpięcie w DriverDom i mount

**Files:**
- Modify: `crates/vertigo/src/driver_module/dom.rs`
- Modify: `crates/vertigo/src/exports.rs`
- Create: `crates/vertigo/src/tests/hydration.rs`
- Modify: `crates/vertigo/src/tests/mod.rs`
- Modify: `crates/vertigo/src/tests/mount_batching.rs` (tylko komentarz modułu)

**Interfaces:**
- Consumes: `reconcile`, `discard`, `split_buffer`, `Reconciled` (Taski 3–5); `api_dom_snapshot` (Task 1).
- Produces: `DriverDom::arm_hydration(&self)`, `DriverDom::flush_hydration(&self)`,
  `DriverDom::inspect_batch(&self, impl Fn(Vec<DriverDomCommand>) + 'static) -> DropResource`.

- [ ] **Step 1: Napisz failujący test przez mount**

Utwórz `crates/vertigo/src/tests/hydration.rs`:

```rust
//! Hydracja od strony `mount`: snapshot wstrzyknięty atrapą, jedna paczka na wyjściu.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode,
    dev::command::DriverDomCommand,
    driver_module::{
        api::api_dom_snapshot,
        driver::get_driver,
        get_driver_dom,
        hydration::{DomSnapshot, SnapshotAttr, SnapshotNode},
    },
    exports::mount,
};

fn snap_element(name: &str, attrs: Vec<(&str, &str)>, children: Vec<u32>) -> SnapshotNode {
    SnapshotNode::Element {
        name: name.to_string(),
        attrs: attrs
            .into_iter()
            .map(|(name, value)| SnapshotAttr {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect(),
        children,
    }
}

/// `<html><head><title>a title</title></head><body><div>hello</div></body></html>`, tak jak
/// wyrenderowałby to serwer.
fn server_rendered() -> DomSnapshot {
    DomSnapshot {
        nodes: vec![
            snap_element("html", vec![], vec![1, 3]),
            snap_element("head", vec![], vec![2]),
            snap_element("title", vec![], vec![5]),
            snap_element("body", vec![], vec![4]),
            snap_element("div", vec![], vec![6]),
            SnapshotNode::Text {
                value: "a title".to_string(),
            },
            SnapshotNode::Text {
                value: "hello".to_string(),
            },
        ],
        head: Some(1),
        body: Some(3),
    }
}

fn app() -> DomNode {
    dom! {
        <html>
            <head>
                <title>"a title"</title>
            </head>
            <body>
                <div>"hello"</div>
            </body>
        </html>
    }
}

/// Montuje i zwraca komendy tak, jak zobaczyłaby je przeglądarka.
///
/// `inspect_batch`, nie `inspect_command`: to drugie odpala się przy kolejkowaniu komendy, a
/// hydracja podmienia zakolejkowany strumień na uzgodniony, więc adopcje nigdy nie przeszłyby
/// tamtą drogą.
fn mount_capturing(init_app: impl FnOnce() -> DomNode) -> Vec<DriverDomCommand> {
    let seen: Rc<RefCell<Vec<DriverDomCommand>>> = Rc::new(RefCell::new(Vec::new()));

    let _tee = get_driver_dom().inspect_batch({
        let seen = seen.clone();
        move |batch| seen.borrow_mut().extend(batch)
    });

    mount(init_app);

    // Patrz `Driver::take_root` - porzucenie drzewa przy zamykaniu wątku sięga do już
    // zwolnionego store'u i przerywa proces.
    drop(get_driver().take_root());

    seen.borrow().clone()
}

/// Cały dokument wyrenderowany przez serwer jest przejmowany, nie odtwarzany.
#[test]
fn a_server_rendered_document_is_adopted_whole() {
    api_dom_snapshot().set_mock(server_rendered());

    let commands = mount_capturing(app);

    let adopted = commands
        .iter()
        .filter(|command| matches!(command, DriverDomCommand::NodeAdopt { .. }))
        .count();

    let created = commands
        .iter()
        .filter(|command| {
            matches!(
                command,
                DriverDomCommand::CreateNode { .. }
                    | DriverDomCommand::CreateText { .. }
                    | DriverDomCommand::CreateComment { .. }
            )
        })
        .count();

    assert_eq!(
        adopted, 4,
        "title, its text, the div and its text: {commands:?}"
    );
    assert_eq!(
        created, 0,
        "nothing should be rebuilt - the roots are resolved by id: {commands:?}"
    );
    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, DriverDomCommand::SnapshotRemove { .. })),
        "there are no leftovers: {commands:?}"
    );
}

/// Bez snapshotu - renderowanie serwerowe, testy na hoście - strumień wychodzi bez zmian.
#[test]
fn without_a_snapshot_the_buffer_goes_out_verbatim() {
    let commands = mount_capturing(app);

    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, DriverDomCommand::NodeAdopt { .. })),
        "nothing to adopt without a snapshot: {commands:?}"
    );

    let created = commands
        .iter()
        .filter(|command| matches!(command, DriverDomCommand::CreateNode { .. }))
        .count();

    assert_eq!(created, 5, "html, head, title, body, div: {commands:?}");
}

/// Resztki serwera, których drzewo klienckie nie chce, idą do usunięcia.
#[test]
fn leftover_server_markup_is_removed() {
    let mut snapshot = server_rendered();

    // Dodatkowy `<footer>` w ciele, którego aplikacja nie rysuje.
    snapshot.nodes.push(snap_element("footer", vec![], vec![]));
    let footer = snapshot.nodes.len() as u32 - 1;

    if let Some(SnapshotNode::Element { children, .. }) = snapshot.nodes.get_mut(3) {
        children.push(footer);
    }

    api_dom_snapshot().set_mock(snapshot);

    let commands = mount_capturing(app);

    let removed: Vec<u32> = commands
        .iter()
        .filter_map(|command| match command {
            DriverDomCommand::SnapshotRemove { snapshot } => Some(*snapshot),
            _ => None,
        })
        .collect();

    assert_eq!(removed, vec![footer]);
}
```

Zarejestruj moduł w `crates/vertigo/src/tests/mod.rs`:

```rust
mod hydration;
```

- [ ] **Step 2: Uruchom testy, żeby sprawdzić, że failują**

Run: `cargo test -p vertigo --all-features tests::hydration`
Expected: FAIL — `no method named arm_hydration` przy kompilacji albo brak `NodeAdopt` w strumieniu

- [ ] **Step 3: Dodaj tryb hydracji do Commands**

Wszystko dzieje się w `struct Commands` w `crates/vertigo/src/driver_module/dom.rs`, bo tam jest bufor i tam jest wysyłka. `DriverDom` dostaje tylko trzy metody delegujące, jak `flush_dom_changes` dzisiaj.

Dodaj dwa pola do `Commands` i zainicjuj je w `Commands::new`:

```rust
    /// Gdy uzbrojony, `flush_dom_changes` nic nie wysyła.
    ///
    /// Flush odpala się przy montowaniu dwukrotnie: raz z hooka `on_after_transaction` po
    /// domknięciu transakcji mount, raz jawnie po `flush_watch`. Porównanie ma dotyczyć
    /// kompletnego drzewa, więc oba te momenty muszą być wyciszone, a wysyłkę robi
    /// `flush_hydration`.
    hydration: ValueMut<bool>,
    /// Podgląd na faktycznie wysyłane paczki. `new_command` odpala się przy kolejkowaniu
    /// komendy, co jest złym momentem dla czegokolwiek, co chce zobaczyć, co dostała
    /// przeglądarka: hydracja podmienia zakolejkowany strumień na uzgodniony.
    new_batch: EventEmitter<Vec<DriverDomCommand>>,
```

Wydziel wysyłkę, żeby obie ścieżki szły przez jedno miejsce — sortowanie i podgląd mają zdarzyć się raz:

```rust
    fn send(&self, commands: Vec<DriverDomCommand>) {
        if commands.is_empty() {
            return;
        }

        let commands = sort_commands(commands);
        self.new_batch.trigger(&commands);
        api_browser_command().dom_bulk_update(commands);
    }

    fn inspect_batch(&self, func: impl Fn(Vec<DriverDomCommand>) + 'static) -> DropResource {
        self.new_batch.add(func)
    }
```

Przepisz `flush_dom_changes` na wyciszenie plus `send`:

```rust
    fn flush_dom_changes(&self) {
        if self.hydration.get() {
            return;
        }

        self.send(self.commands.take());
    }
```

Dopisz uzbrojenie i domknięcie:

```rust
    fn arm_hydration(&self) {
        self.hydration.set(true);
    }

    /// Kończy montowanie: pobiera snapshot, uzgadnia bufor ze stanem przeglądarki, wysyła
    /// i rozbraja tryb. Od tego momentu wszystko wraca do zwykłego flushowania po
    /// transakcji.
    fn flush_hydration(&self) {
        self.hydration.set(false);

        let commands = self.commands.take();

        if commands.is_empty() {
            return;
        }

        let commands = match api_dom_snapshot().get() {
            Some(snapshot) => {
                if hydration_disabled() {
                    discard(split_buffer(commands), &snapshot)
                } else {
                    let Reconciled { commands, report } =
                        reconcile(split_buffer(commands), &snapshot);
                    report.publish();
                    commands
                }
            }
            None => commands,
        };

        self.send(commands);
    }
```

W `impl DriverDom` dopisz trzy delegacje obok istniejącej `flush_dom_changes`:

```rust
    pub(crate) fn arm_hydration(&self) {
        self.commands.arm_hydration();
    }

    pub(crate) fn flush_hydration(&self) {
        self.commands.flush_hydration();
    }

    pub fn inspect_batch(&self, func: impl Fn(Vec<DriverDomCommand>) + 'static) -> DropResource {
        self.commands.inspect_batch(func)
    }
```

Dopisz pomocniczą funkcję w tym samym pliku:

```rust
/// Flaga z wiersza poleceń `--disable-hydration`, wstawiana przez cli jako
/// `data-env-disable-hydration` i czytana tą samą drogą co pozostałe zmienne środowiskowe.
///
/// Polityka należy do rusta: js zwraca snapshot niezależnie od flagi, a decyzję o tym, czy
/// cokolwiek adoptować, podejmuje ta gałąź.
fn hydration_disabled() -> bool {
    api_browser_command().get_env("disable-hydration") == Some("true".to_string())
}
```

Dopisz importy: `api_dom_snapshot` z `crate::driver_module::api`, `discard`, `reconcile`, `split_buffer` i `Reconciled` z `crate::driver_module::hydration`, oraz `ValueMut` z `crate::dev` (dotąd ten plik go nie potrzebował).

Dla orientacji: `exec_command(command: CommandForBrowser) -> JsJson` i `get_env(&self, name: impl Into<String>) -> Option<String>` — obie są już w `api_browser_command.rs` i nie wymagają zmian.

- [ ] **Step 4: Podepnij to w mount**

W `crates/vertigo/src/exports.rs` zamień końcówkę `mount`:

```rust
pub(crate) fn mount(init_app: impl FnOnce() -> DomNode) {
    init_env();

    // Before the transaction, deliberately: `get_driver()` is what registers the flush hook,
    // and the transaction below only suppresses mid-build flushes if it is the outermost one.
    let driver = get_driver();

    // Nothing reaches the browser until the tree is complete: hydration compares the whole
    // thing against the document the server rendered, so a partial flush would have it
    // matching half a tree.
    get_driver_dom().arm_hydration();

    driver.transaction(|_| {
        let root_view = init_app();
        driver.set_root(root_view);
    });

    // `flush_watch` - and so `when_connect` - runs *after* the hooks, so anything it queued
    // is still sitting in the buffer. This is where the batch is reconciled and sent.
    get_driver_dom().flush_hydration();
}
```

- [ ] **Step 5: Uruchom testy**

Run: `cargo test -p vertigo --all-features tests::hydration`
Expected: PASS (3 testy)

- [ ] **Step 6: Sprawdź, że istniejące testy mountu nie zmieniły znaczenia**

Run: `cargo test --all-features`
Expected: PASS, w tym `tests::mount_batching` i `tests::dom_command_counts` **bez zmian w kodzie**

Warto wiedzieć, czemu te testy nie ruszają się z miejsca, bo to mówi, co dokładnie zmieniłeś. `mount_capturing_batches` podgląda strumień przez `inspect_command`, które odpala się przy kolejkowaniu komendy, i cięcia stawia na odpaleniu hooka po transakcji — a obu tych momentów wyciszenie flushowania nie dotyczy. Na hoście nie ma snapshotu, więc bufor wychodzi bez zmian i liczby komend są te same.

To znaczy, że `mount_batching` po tej zmianie mierzy trochę inną rzecz niż zapowiada: jedna paczka nie wynika już tylko z transakcji, ale przede wszystkim z wyciszenia. Zaktualizuj komentarz modułu na początku pliku — właściwość, na której zależy hydracji, nazywa się teraz inaczej i jest mocniejsza: nic nie wychodzi do przeglądarki, dopóki drzewo nie jest kompletne, bez względu na to, co się dzieje z transakcjami. Nie dopisuj nowego testu; własność sprawdza test `a_server_rendered_document_is_adopted_whole` z Taska 6, który zobaczyłby częściowe drzewo jako brakujące adopcje.

Jeśli którykolwiek z tych testów **jednak** failuje, nie poprawiaj oczekiwań — to znaczy, że wyciszenie przecieka albo `send` woła się dwa razy.

- [ ] **Step 7: Uruchom całość**

Run: `cargo test --all-features && cargo clippy --locked -p vertigo --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add crates/vertigo/src/driver_module/dom.rs crates/vertigo/src/exports.rs crates/vertigo/src/tests
git commit -m "feat(hydration): reconcile the mount batch against the browser snapshot"
```

---

### Task 7: Strona JavaScriptu — builder snapshotu, dwie komendy, kasacja hydration.ts

**Files:**
- Create: `crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.ts`
- Create: `crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.test.ts`
- Delete: `crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.ts`
- Delete: `crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.test.ts`
- Modify: `crates/vertigo/src/driver_module/src_js/api/api.ts`
- Modify: `crates/vertigo/src/driver_module/src_js/api/command/dom/dom.ts`
- Modify: `crates/vertigo/src/driver_module/src_js/api/command/dom/map_nodes.ts`
- Modify: `rollup.test.config.mjs`
- Modify: `package.json`

**Interfaces:**
- Consumes: `Tag.NodeAdopt`, `Tag.SnapshotRemove` (Task 2); kształt `DomSnapshot` z Taska 1.
- Produces: `buildSnapshot(document: Document): SnapshotResult` gdzie `SnapshotResult = { payload: { nodes: Array<SnapshotNodeJson>, head: number | null, body: number | null }, nodes: Array<Node> }`; `DriverDom.snapshot(): JsJsonType`.

- [ ] **Step 1: Napisz failujące testy buildera**

Utwórz `crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.test.ts`.

W repozytorium nie ma `jsdom` i nie dokładamy go — `hydration.test.ts` miał własne atrapy i to je przenosimy. Skopiuj z niego klasy `MockNode`, `MockElement` i `MockSvgElement` (są na początku pliku, wraz z komentarzem wyjaśniającym, czym różni się wielkość liter w svg) i dopisz `MockText` oraz `MockComment`, jeśli ich tam nie ma.

Atrapy nie są instancjami `Element` ani `Text`, więc builder rozpoznaje rodzaj węzła po `nodeType` — tak samo jak prawdziwy DOM, tylko bez `instanceof`.

```ts
import { buildSnapshot } from './snapshot';

// --- MOCKS: przeniesione z hydration.test.ts ---
// (MockNode, MockElement, MockSvgElement — skopiuj jeden do jednego)

class MockText extends MockNode {
    data: string;
    constructor(data: string) {
        super(MockNode.TEXT_NODE);
        this.data = data;
    }
}

class MockComment extends MockNode {
    static COMMENT_NODE = 8;
    data: string;
    constructor(data: string) {
        super(MockComment.COMMENT_NODE);
        this.data = data;
    }
}

const assert = (condition: boolean, message: string) => {
    if (!condition) {
        throw new Error(message);
    }
};

const elementName = (node: any): string => ('Element' in node ? node.Element.name : '');

const names = (payload: any): Array<string> =>
    payload.nodes.filter((node: any) => 'Element' in node).map(elementName);

/// `<html lang="pl"><head><title>a</title></head><body><div class="x">hi</div></body></html>`
const document = () => {
    const html = new MockElement('html');
    html.setAttribute('lang', 'pl');

    const head = new MockElement('head');
    const title = new MockElement('title');
    title.appendChild(new MockText('a'));
    head.appendChild(title);

    const body = new MockElement('body');
    const div = new MockElement('div');
    div.setAttribute('class', 'x');
    div.appendChild(new MockText('hi'));
    body.appendChild(div);

    html.appendChild(head);
    html.appendChild(body);

    return html;
};

const run = () => {
    {
        const { payload, nodes } = buildSnapshot(document() as any);

        assert(payload.nodes.length === nodes.length, 'the payload and the node table must line up');
        assert(payload.head === 1, `head should be index 1, got ${payload.head}`);
        assert(payload.body === 3, `body should be index 3, got ${payload.body}`);

        const root = payload.nodes[0];
        assert(elementName(root) === 'html', 'index 0 is <html>');
        assert(
            'Element' in root && root.Element.attrs.some(attr => attr.name === 'lang' && attr.value === 'pl'),
            'attributes travel with the element',
        );
        assert(
            'Element' in root && root.Element.children.length === 2,
            'children are recorded as indices into the same list',
        );
        assert('Text' in payload.nodes[2], 'the title text is at index 2');
    }

    {
        // tagName jest wielkimi literami dla html i zachowuje wielkość liter dla svg; rust
        // porównuje bez wielkości liter, więc wysyłamy wszystko małymi.
        const html = new MockElement('html');
        const body = new MockElement('body');
        const svg = new MockSvgElement('svg');
        svg.appendChild(new MockSvgElement('linearGradient'));
        body.appendChild(svg);
        html.appendChild(body);

        const { payload } = buildSnapshot(html as any);

        assert(names(payload).includes('svg'), `expected svg among ${names(payload).join()}`);
        assert(
            names(payload).includes('lineargradient'),
            `expected a lowercased svg name among ${names(payload).join()}`,
        );
    }

    {
        // Białe znaki z formatowania jadą do rusta - js nie może ich bezpiecznie odfiltrować,
        // bo w <pre> i przy treści inline są znaczące. Decyzja należy do rusta.
        const html = new MockElement('html');
        const body = new MockElement('body');
        body.appendChild(new MockText('\n  '));
        body.appendChild(new MockElement('div'));
        html.appendChild(body);

        const { payload } = buildSnapshot(html as any);

        const texts = payload.nodes.filter(node => 'Text' in node);
        assert(texts.length === 1, `whitespace text nodes must be sent, got ${texts.length}`);
    }

    {
        // Komentarze też, choćby po to, żeby rust wiedział, że coś zajmuje miejsce.
        const html = new MockElement('html');
        const body = new MockElement('body');
        body.appendChild(new MockComment('hand written'));
        html.appendChild(body);

        const { payload } = buildSnapshot(html as any);

        assert(
            payload.nodes.some(node => 'Comment' in node),
            'comments must be sent',
        );
    }

    {
        // Skrypt ładujący wasm nie jest częścią drzewa aplikacji.
        const html = new MockElement('html');
        const body = new MockElement('body');
        body.appendChild(new MockElement('div'));
        const script = new MockElement('script');
        script.setAttribute('data-vertigo-run-wasm', 'x');
        body.appendChild(script);
        html.appendChild(body);

        const { payload, nodes } = buildSnapshot(html as any);

        assert(!names(payload).includes('script'), `the loader script must be skipped, got ${names(payload).join()}`);
        assert(payload.nodes.length === nodes.length, 'skipping must not desynchronise the two lists');
    }

    console.info('snapshot.test.ts: ok');
};

run();
```

- [ ] **Step 2: Zarejestruj test i wypisz stary**

W `rollup.test.config.mjs` podmień pierwszy wpis (`hydration.test.ts` → `build/hydration.test.js`) na:

```js
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/snapshot.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
```

W `package.json` podmień skrypt:

```json
    "test": "rollup -c rollup.test.config.mjs && node build/snapshot.test.js && node build/dom_wire.test.js && node build/fetchExec.test.js"
```

- [ ] **Step 3: Uruchom testy, żeby sprawdzić, że failują**

Run: `npm run test`
Expected: FAIL — `Cannot find module './snapshot'`

- [ ] **Step 4: Napisz builder**

Utwórz `crates/vertigo/src/driver_module/src_js/api/command/dom/snapshot.ts`:

```ts
import { JsJsonType } from '../../../jsjson';

type SnapshotNodeJson =
    | { Element: { name: string, attrs: Array<{ name: string, value: string }>, children: Array<number> } }
    | { Text: { value: string } }
    | { Comment: { value: string } };

interface SnapshotPayload {
    nodes: Array<SnapshotNodeJson>;
    head: number | null;
    body: number | null;
}

export interface SnapshotResult {
    /// Idzie do rusta.
    payload: SnapshotPayload;
    /// Zostaje tutaj: indeks w tej tablicy jest adresem, którym rust adresuje węzeł w
    /// komendach NodeAdopt i SnapshotRemove. Obie listy rosną razem, więc pozycje się
    /// zgadzają.
    nodes: Array<Node>;
}

const ELEMENT_NODE = 1;
const TEXT_NODE = 3;
const COMMENT_NODE = 8;

/// Czy ten węzeł jest infrastrukturą, a nie treścią aplikacji.
///
/// Div z metadanymi odczepia od dokumentu konstruktor `Metadata`, jeszcze przed bootem wasma,
/// więc tutaj go nie widać. Skrypt ładujący trzeba pominąć jawnie.
const isInfrastructure = (node: Element): boolean => node.hasAttribute('data-vertigo-run-wasm');

/// Przechodzi drzewo w kolejności pre-order i buduje płaską listę.
///
/// Płaską, nie zagnieżdżoną, bo przy tym samym przejściu zapisujemy węzły do tablicy - i
/// wtedy indeks w liście jest jednocześnie adresem, pod którym w czasie stałym znajdziemy
/// prawdziwy węzeł, gdy przyjdzie komenda adopcji.
///
/// Rodzaj węzła rozpoznawany jest przez `nodeType`, nie `instanceof`: to samo, co robi
/// prawdziwy DOM, a przy okazji jedyne, co potrafią atrapy z testu.
///
/// Bierze węzeł korzenia, a nie `Document`, z tego samego powodu.
export const buildSnapshot = (root: Node): SnapshotResult => {
    const payload: SnapshotPayload = { nodes: [], head: null, body: null };
    const nodes: Array<Node> = [];

    const visit = (node: Node, depth: number): number | null => {
        if (node.nodeType === ELEMENT_NODE) {
            const element = node as Element;

            if (isInfrastructure(element)) {
                return null;
            }

            // Rust porównuje bez uwzględniania wielkości liter, co obsługuje html i svg
            // naraz - dzięki temu tablica SVG_TAGS nie musi trafić do wasma.
            const name = element.tagName.toLowerCase();

            const index = payload.nodes.length;
            payload.nodes.push({
                Element: {
                    name,
                    attrs: element.getAttributeNames().map(attribute => ({
                        name: attribute,
                        value: element.getAttribute(attribute) ?? '',
                    })),
                    children: [],
                },
            });
            nodes.push(node);

            // Jedyne dwa elementy, których id rust zna z góry. Rozpoznawane po nazwie na
            // pierwszym poziomie, bo dokument ma dokładnie jeden `<head>` i jeden `<body>`,
            // a porównanie z `document.head` nie zadziałałoby dla atrap.
            if (depth === 1) {
                if (name === 'head') {
                    payload.head = index;
                }
                if (name === 'body') {
                    payload.body = index;
                }
            }

            const children: Array<number> = [];
            for (const child of Array.from(node.childNodes)) {
                const childIndex = visit(child, depth + 1);
                if (childIndex !== null) {
                    children.push(childIndex);
                }
            }

            const entry = payload.nodes[index];
            if (entry !== undefined && 'Element' in entry) {
                entry.Element.children = children;
            }

            return index;
        }

        if (node.nodeType === TEXT_NODE || node.nodeType === COMMENT_NODE) {
            const index = payload.nodes.length;
            const value = (node as Text | Comment).data;

            // Teksty z samych białych znaków też: w `<pre>` i przy treści inline są
            // znaczące, więc js nie może ich bezpiecznie odfiltrować.
            payload.nodes.push(
                node.nodeType === TEXT_NODE ? { Text: { value } } : { Comment: { value } },
            );
            nodes.push(node);

            return index;
        }

        return null;
    };

    visit(root, 0);

    return { payload, nodes };
};

export const snapshotToJson = (payload: SnapshotPayload): JsJsonType => payload as unknown as JsJsonType;
```

- [ ] **Step 5: Uruchom testy JS**

Run: `npm run test`
Expected: PASS

- [ ] **Step 6: Podepnij builder i dwie komendy w DriverDom**

W `crates/vertigo/src/driver_module/src_js/api/command/dom/dom.ts`:

Dodaj pole i metodę:

```ts
    private snapshotNodes: Array<Node> | null = null;

    /// Odpowiedź na `DomSnapshotGet`. Tablica węzłów zostaje tutaj - rust adresuje je
    /// indeksem.
    public snapshot = (): JsJsonType => {
        const { payload, nodes } = buildSnapshot(document.documentElement);
        this.snapshotNodes = nodes;
        return snapshotToJson(payload);
    }
```

W gorącej ścieżce `update`, w `switch` po tagu, dodaj dwie gałęzie:

```ts
                case Tag.NodeAdopt: {
                    const id = cursor.varint();
                    const snapshot = cursor.varint();
                    const node = this.snapshotNodes?.[snapshot];

                    if (node === undefined) {
                        console.error(`NodeAdopt: no snapshot node at ${snapshot}`);
                        break;
                    }

                    this.nodes.set(id, node as Element | Comment | Text);

                    if (node.nodeType === 1) {
                        // Bez tego przestaje działać przechwytywanie kliknięć w linki -
                        // dotychczas robił to `claimNode` w gałęzi hydracji.
                        injects(node as Element, this.appLocation);
                    }
                    break;
                }
                case Tag.SnapshotRemove: {
                    const snapshot = cursor.varint();
                    const node = this.snapshotNodes?.[snapshot];

                    if (node !== undefined) {
                        (node as ChildNode).remove();
                    }
                    break;
                }
```

Na końcu `update`, po zastosowaniu komend, zwolnij tablicę i usuń wywołanie `removeInitNodes`:

```ts
        // Paczka hydracyjna jest jedyną, która adresuje węzły snapshotu.
        this.snapshotNodes = null;
```

Usuń z `update` gałąź pierwszej paczki (wywołanie `hydrate` i warunek z `hasInitNodes`/`getEnabledHydration`) oraz wywołanie `this.nodes.removeInitNodes()`.

Z `createNode` i `createText` usuń strażników `if (this.nodes.has(id)) { return; }` — rust nie emituje już tworzenia dla węzłów adoptowanych, więc są martwym kodem. Strażniki na id 1, 2 i 3 **zostaw**.

Dopisz importy: `buildSnapshot` i `snapshotToJson` z `./snapshot`, oraz `injects` z `./injects` — dziś `dom.ts` importuje z tego modułu tylko `hydrateLink`, bo `injects` wołało `hydration.ts`.

W `crates/vertigo/src/driver_module/src_js/api/api.ts` `DomSnapshotGet` jest wariantem bez pól, więc na drucie jest samym napisem — tak jak `FetchCacheGet`. Dopisz go do unii `ExecType`:

```ts
type ExecType
    = 'FetchCacheGet'
    | 'DomSnapshotGet'
    | 'IsBrowser'
```

i dodaj gałąź w `exec`, obok tej dla `FetchCacheGet`:

```ts
        if (safeArg === 'DomSnapshotGet') {
            return this.dom.snapshot();
        }
```

- [ ] **Step 7: Usuń hydrację z JS**

```bash
git rm crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.ts crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.test.ts
```

W `crates/vertigo/src/driver_module/src_js/api/command/dom/map_nodes.ts` usuń: pole `initNodes`, jego inicjalizację w konstruktorze, metody `removeInitNodes`, `hasInitNodes` i `claimNode`. Zostaw `set`, `getAnyOption`, `insertBefore`, `insertCss`, `addStyles` i gettery.

`Metadata.getEnabledHydration` przestaje być używane w JS — flagę czyta teraz rust przez `get_env`. Usuń tę metodę.

- [ ] **Step 8: Zbuduj bundel i uruchom testy**

Run: `npx tsc --noEmit -p tsconfig.json && npx rollup -c && npm run test`
Expected: PASS. `npx rollup -c` nadpisuje `crates/vertigo/src/driver_module/wasm_run.js` — ten plik jest w repozytorium i musi wejść do commita.

- [ ] **Step 9: Commit**

```bash
git add crates/vertigo/src/driver_module/src_js crates/vertigo/src/driver_module/wasm_run.js rollup.test.config.mjs package.json
git commit -m "refactor(hydration): js builds the snapshot and applies adopt commands"
```

---

### Task 8: Testy przeglądarkowe i changelog

**Files:**
- Modify: `tests/demo/ssr.rs`
- Modify: `docs/CHANGELOG.md`

**Interfaces:**
- Consumes: `HydrationReport` publikowany na `window.__vertigo_hydration` (Task 4), pełny przebieg z Tasków 6 i 7.

- [ ] **Step 1: Sprawdź, co widzi test przeglądarkowy**

Nazwy pól raportu są zachowane (`rootFound` przez `#[js_json(rename)]`, pozostałe bez zmian), więc `hydration_report` w `tests/demo/ssr.rs` powinien działać bez zmian. Zaktualizuj tylko komentarz dokumentacyjny nad `struct HydrationReport` i nad `hydration_report`, żeby mówił, że raport powstaje w ruście i jest parkowany na `window` przez `dom_access`, a nie liczony w JS.

- [ ] **Step 2: Uruchom testy przeglądarkowe**

Wymagany działający WebDriver na `localhost:9515`. Uruchom go w osobnym terminalu (`chromedriver --port=9515`), potem:

Run: `cargo test --package fantoccini-tests --test demo -- --ignored --nocapture`
Expected: PASS, w szczególności `hydration_is_complete` na trasach `""` i `svg`

Jeśli `matched < hydratable`, przeczytaj wypisany raport i porównaj z regułami zliczania: markery komentarzy i teksty poza pierwszym w scalonym ciągu idą do `skipped`, nie do `hydratable`. Rozbieżność znaczy, że któraś z tych dwóch kategorii jest liczona podwójnie.

- [ ] **Step 3: Zmierz rozmiar startowej paczki**

To kryterium sukcesu numer 2 ze specyfikacji. Wejdź na stronę demo z uruchomionym serwerem (`task demo`) i w konsoli przeglądarki odczytaj raport:

```js
window.__vertigo_hydration
```

`matched` powinno być równe `hydratable`, a liczba tworzonych węzłów w paczce — bliska zeru. Zapisz wynik w opisie commita.

- [ ] **Step 4: Uruchom pozostałe testy przeglądarkowe**

Run: `cargo test --package fantoccini-tests --test basic --test demo -- --ignored`
Expected: PASS

- [ ] **Step 5: Dopisz wpis do changeloga**

W `docs/CHANGELOG.md`, w sekcji niewydanej wersji, dopisz w stylu istniejących wpisów:

```markdown
- Hydration moved from JavaScript to Rust. The browser is asked for a DOM snapshot
  (`DomSnapshotGet`), and the mount batch is reconciled against it in wasm: existing nodes
  are adopted through the new `NodeAdopt` command and only differences are sent. Two
  consequences are visible from the outside: the first DOM batch of a server-rendered page is
  much smaller, and a merged run of adjacent text nodes now keeps every node bound to its id
  (previously all but the first were left dangling).
```

- [ ] **Step 6: Uruchom pełne ci**

Run: `task ci`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add tests/demo/ssr.rs docs/CHANGELOG.md
git commit -m "test(hydration): browser coverage and changelog for rust-side hydration"
```

---

## Notatki dla wykonawcy

**Czego nie ruszać.** `tags.ts` zostaje w JS — `createElement` potrzebuje `SVG_TAGS` do wyboru przestrzeni nazw, i tylko `expectedTagName` przestaje być używane (usuń je, jeśli nic go nie importuje). `decodeCommands` w `dom_wire.ts` zostaje, ale wyłącznie jako narzędzie dla `dom_wire.test.ts`; nie importuj go z gorącej ścieżki.

**Czego się spodziewać przy pożyczkach.** `Matcher` trzyma `tree` i `snapshot` jako referencje o czasie życia `'a`. Wywołanie `self.tree.get(..)` pożycza `self`, co zderza się z `self.out.push(..)`. Rozwiązanie w każdym takim miejscu: `let tree = self.tree;` na początku metody — kopiuje samą referencję, której czas życia nie jest związany z `self`.

**Rekurencja.** `reconcile_children` i `create_subtree` są rekurencyjne po głębokości drzewa. Dla realnych dokumentów to kilkadziesiąt poziomów; nie ma powodu przepisywać tego na pętlę.

**Kolejność ma znaczenie w dwóch miejscach.** Adopcje muszą wyjść przed czymkolwiek, co odwołuje się do adoptowanych identyfikatorów — dlatego `emit_children` emituje `NodeAdopt` przed zejściem w głąb. Usunięcia muszą wyjść na końcu — tym zajmuje się `sort_commands`, do którego dopisujesz `SnapshotRemove` w Tasku 2.
