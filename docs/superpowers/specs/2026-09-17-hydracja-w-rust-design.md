# Hydracja po stronie Rusta — projekt

Data: 2026-09-17

## 1. Cel

Przenieść decyzje hydracyjne z JavaScriptu do Rusta. Warstwa JS ma zostać
wykonawcą komend: potrafi odczytać DOM i zastosować komendę, ale nie podejmuje
żadnych decyzji o tym, który węzeł przeglądarki odpowiada któremu węzłowi
wygenerowanemu przez vertigo.

Motywacją jest **cieńsza warstwa JS** — mniej kodu i mniej bundla. Nie jest nią
naprawa dopasowania ani nowa funkcjonalność. Tam, gdzie przeprowadzka daje
poprawność „za darmo" (scalone węzły tekstowe, patrz 9.3), bierzemy to jako
efekt uboczny, a nie jako cel.

## 2. Stan obecny

Hydracja jest dziś w całości w JS, w `crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.ts`
(373 linie). Przebieg:

1. Rust montuje aplikację w jednej transakcji (`mount` w `crates/vertigo/src/exports.rs`)
   i wysyła **jedną** paczkę `DomBulkUpdate` tworzącą całe drzewo od zera.
2. JS na pierwszej paczce odtwarza z niej wirtualne drzewo (`createVirtualNodes`),
   czyli rekonstruuje strukturę ze strumienia komend.
3. Chodzi po `document.body` i `document.head`, dopasowuje węzły heurystycznie
   (nazwa tagu przez `expectedTagName`, atrybuty w obie strony, tekst po `trim`),
   „zaklepuje" trafienia w `MapNodes.claimNode` i usuwa niedopasowane.
4. Ten sam bufor jest następnie aplikowany normalnie; `createNode`/`createText`
   pomijają id, które już są w mapie.
5. `MapNodes.removeInitNodes()` usuwa węzły SSR, których nikt nie zaklepał.

Rust nie wie o DOM przeglądarki nic. Nie istnieje żadne wywołanie JS → Rust
przekazujące stan dokumentu.

### 2.1. Fakty z kodu, które kształtują ten projekt

- **Rust nie zna zawartości swojego drzewa.** `DomElement` trzyma tylko
  `id_dom`, dzieci, subskrypcje i `class_state`. Atrybuty są zapisywane wprost
  do drivera przez domknięcie w `add_attr` i nigdzie nie zostają. `DomText`
  trzyma `id_dom` i subskrypcje — treść tekstu nie jest przechowywana.
  Wyjątkiem jest `class`, którego wartość trzyma `ClassState`, ale z innego
  powodu: żeby scalić atrybut z `css={}`. `get_children()` istnieje wyłącznie
  pod `#[cfg(test)]`.

  **Konsekwencja:** opisem wyrenderowanego drzewa jest strumień komend, nie
  drzewo `DomNode`. Kod jest już wokół tego ukształtowany — komentarz w
  `DomText::patched` mówi wprost, że węzeł tworzony jest od razu z wartością,
  bo hydracja czyta `CreateText`.

- **Przy starcie nie ma na co czekać.** SSR wstrzykuje wyniki fetchów do HTML
  jako `data-fetch-cache` na `#v-metadata` (`html_build_response.rs`).
  `LazyCache::new` odczytuje je **synchronicznie w konstruktorze** i ustawia
  `Resource::Ready`; `needs_update()` zwraca wtedy `false`, bo `expiry` liczone
  jest jako `now() + ttl` już w przeglądarce. Nie powstaje żaden future ani
  `spawn`. Drzewo zbudowane w transakcji mount jest kompletne.

  Asynchronicznie dorasta tylko to, czego SSR nie pokrył (fetch za
  `is_browser()`, inny klucz `SsrFetchRequest`, przekroczony 10-sekundowy
  timeout SSR, rewalidacja `when_connect`, ręczne `spawn`). W tych przypadkach
  w HTML nie ma odpowiadającego węzła do reużycia, więc czekanie i tak nie
  pomogłoby — a kosztowałoby odroczenie interaktywności, bo do pierwszej paczki
  nie ma podpiętych callbacków.

  **Konsekwencja:** „ustabilizowany stan" to domknięcie transakcji mount wraz
  z `flush_watch`. Nie budujemy mechanizmu wykrywania ciszy w pętli zdarzeń
  (w `vertigo` takiego nie ma — `spawn_local` tworzy osobny `Rc<Task>` bez
  wspólnej kolejki).

- **Produkcyjny SSR renderuje z formatowaniem.** `html_build_response.rs` woła
  `root_html.convert_to_string(true)`, więc w HTML są wcięcia i nowe linie,
  które parser zamienia na węzły tekstowe nieistniejące w drzewie vertigo.
  Wyjątkiem są konteksty preformatowane i inline, gdzie `convert_to_string`
  przechodzi na `Format::none()` i nie wstrzykuje niczego.

- **SSR wycina komentarze.** `get_render_child_mode` odrzuca `HtmlNode::Comment`,
  więc markery `render_value`/`render_list` nie mają w HTML odpowiednika.

- **SSR nie emituje identyfikatorów.** Produkcja używa `with_id: false`;
  `data-id` pojawia się tylko w testach.

- **Id 1, 2, 3 są stałe** dla `html`, `head` i `body` (`DomId::from_name`).
  `MapNodes.getAnyOption` rozwiązuje je dynamicznie i `MapNodes.set` je ignoruje,
  więc nigdy nie wymagają wiązania.

- **Format druciarski jest dopisywalny na koniec.** `command_wire.rs` dokumentuje
  tagi 1–13 jako append-only.

## 3. Zakres

W zakresie:

- pobranie snapshotu DOM z przeglądarki do Rusta,
- uzgodnienie drzewa docelowego ze snapshotem po stronie Rusta,
- emisja zredukowanego strumienia komend z adopcją istniejących węzłów,
- usunięcie logiki hydracyjnej z JS,
- testy jednostkowe algorytmu w Ruście.

Poza zakresem:

- czekanie na operacje asynchroniczne przed hydracją (patrz 2.1),
- emitowanie identyfikatorów węzłów do HTML przez SSR (`data-v-id`) — rozważone
  i odrzucone, patrz 15.2,
- przechowywanie atrybutów i tekstów w drzewie `DomNode` — rozważone i odrzucone,
  patrz 15.1,
- hydracja częściowa, wyspy, cokolwiek nowego w API aplikacji.

## 4. Architektura — przepływ

```
JS: boot wasma  →  vertigo_entry_function  →  Rust: mount()
                                                 │
                    arm_hydration()  ────────────┤  wycisza flush
                                                 │
                    transaction { init_app(); set_root() }
                    flush_watch()                │  bufor komend rośnie,
                                                 │  nic nie wychodzi
                    flush_hydration()  ──────────┤
                        │                        │
                        ├─ CommandForBrowser::DomSnapshotGet  ──→ JS buduje
                        │                                          snapshot
                        │   ←── DomSnapshot | Null ────────────────┘
                        │
                        ├─ podział bufora (7.1)
                        ├─ budowa indeksu docelowego (8)
                        ├─ dopasowanie (9)
                        ├─ emisja strumienia (10)
                        └─ DomBulkUpdate  ──→ JS: aplikacja komend
```

Tryb hydracji to jeden bit w `DriverDom`, uzbrajany **przed** transakcją.
Dopóki jest uzbrojony, `flush_dom_changes()` wraca natychmiast bez wysyłki.
Jest to konieczne, bo flush odpala się dziś dwukrotnie: raz z hooka
`on_after_transaction` po domknięciu transakcji mount, raz jawnie po
`flush_watch`. Porównanie ma dotyczyć kompletnego drzewa, więc oba te momenty
muszą być wyciszone.

```rust
pub(crate) fn mount(init_app: impl FnOnce() -> DomNode) {
    init_env();
    let driver = get_driver();

    get_driver_dom().arm_hydration();

    driver.transaction(|_| {
        let root_view = init_app();
        driver.set_root(root_view);
    });

    get_driver_dom().flush_hydration();
}
```

`flush_hydration()` pobiera snapshot, uzgadnia bufor, wysyła i rozbraja tryb.
Po rozbrojeniu wszystko wraca do dzisiejszego zachowania: każda transakcja
flushuje swoje komendy hookiem.

## 5. Handshake

Rust **pyta** o snapshot; JS go nie wypycha. Nowy wariant:

```rust
CommandForBrowser::DomSnapshotGet
```

Odpowiedź: `JsJson` dekodowany do `Option<DomSnapshot>` (`JsJson::Null` znaczy
„brak snapshotu").

Uzasadnienie kierunku:

- Kolejność przestaje być umową między JS a Rustem. Nie da się pominąć
  wywołania w niestandardowym loaderze, bo Rust pyta wtedy, kiedy potrzebuje.
- Nie dotykamy sygnatury eksportu `vertigo_entry_function`, która służy właśnie
  do wykrywania rozjazdu wersji JS i wasma.
- Istnieje precedens o udokumentowanej korzyści: `api_fetch_cache.rs` pobiera
  fetch cache leniwie, tą samą drogą, żeby aplikacja, która z mechanizmu nie
  korzysta, nie wciągała do wasma jego dekodera.

Snapshot pobieramy dopiero w `flush_hydration()`, nie na starcie. Nie ma powodu
trzymać go w pamięci przez cały czas budowy drzewa, a DOM jest w tym momencie
nadal nietknięty, bo nie wyszła jeszcze ani jedna komenda.

## 6. Format snapshotu

Płaska lista w kolejności pre-order, nie drzewo zagnieżdżone. Przy tym samym
przejściu JS zapisuje węzły do tablicy `snapshotNodes: Node[]`, więc indeks
w liście jest jednocześnie adresem, pod którym JS w czasie stałym znajdzie
prawdziwy węzeł, gdy przyjdzie komenda adopcji lub usunięcia.

```rust
#[derive(AutoJsJson)]
pub struct DomSnapshot {
    /// Indeks w tej liście jest adresem węzła w protokole (7.2).
    /// Element 0 to <html>.
    nodes: Vec<SnapshotNode>,
    head: Option<u32>,
    body: Option<u32>,
}

#[derive(AutoJsJson)]
pub enum SnapshotNode {
    Element {
        /// tagName zmniejszone do małych liter (patrz 9.1).
        name: String,
        attrs: Vec<SnapshotAttr>,
        children: Vec<u32>,
    },
    Text { value: String },
    Comment { value: String },
}

#[derive(AutoJsJson)]
pub struct SnapshotAttr {
    name: String,
    value: String,
}
```

`SnapshotAttr` jest osobną strukturą, a nie krotką, bo nie polegamy na obsłudze
krotek w `AutoJsJson`.

`head` i `body` przychodzą jako jawne indeksy, żeby Rust nie odgadywał ich po
nazwie tagu.

### 6.1. Co wchodzi do snapshotu

- Cały dokument od `document.documentElement` w pre-order.
- **Wszystkie** węzły tekstowe, również złożone z samych białych znaków. JS nie
  może ich bezpiecznie odfiltrować, bo w `<pre>` i przy treści inline białe
  znaki są znaczące. Decyzja należy do Rusta (9.2).
- Komentarze — żeby „usuwanie resztek" umiało je objąć.

### 6.2. Co nie wchodzi

- `#v-metadata` — wypada samo, bo `Metadata` odczepia go od dokumentu
  w konstruktorze, jeszcze przed bootem wasma.
- `<script data-vertigo-run-wasm>` — pomijany jawnie po atrybucie.

To drobna zmiana zachowania: dziś `removeInitNodes()` usuwa ten skrypt z drzewa,
po zmianie zostanie na stronie. Nieszkodliwe i oszczędza mieszania w DOM.

### 6.3. Cykl życia `snapshotNodes`

Tablica powstaje przy obsłudze `DomSnapshotGet` i jest zwalniana na koniec
najbliższego wywołania `DriverDom.update`, czyli po zastosowaniu paczki
hydracyjnej. Jest to dwulinijkowe zarządzanie czasem życia, bez żadnej wiedzy
o hydracji.

## 7. Protokół

### 7.1. Podział bufora

Komendy zebrane w czasie mount rozchodzą się na dwie grupy.

**Pochłaniane przez indeks i generowane od nowa:** `CreateNode`, `CreateText`,
`CreateComment`, `SetAttr`, `RemoveAttr`, `UpdateText`, `InsertBefore`,
`RemoveNode`, `RemoveText`, `RemoveComment`.

**Przechodzące bez zmian:** `InsertCss`, `CallbackAdd`, `CallbackRemove`.
CSS nie ma związku z tożsamością węzłów, a callbacki są kluczowane przez
`DomId`, który zachowujemy — jest im więc obojętne, czy węzeł został adoptowany
czy utworzony.

### 7.2. Nowe komendy

Dwie, dopisane na koniec formatu druciarskiego:

```rust
/// Tag 14. Wiąże istniejący węzeł przeglądarki z identyfikatorem vertigo.
NodeAdopt { id: DomId, snapshot: u32 },

/// Tag 15. Usuwa węzeł snapshotu, który nie został adoptowany.
SnapshotRemove { snapshot: u32 },
```

`SnapshotRemove` jest potrzebne, bo `RemoveNode` adresuje węzeł przez `DomId`,
a resztki SSR żadnego `DomId` nie mają — nigdy nie zostały zarejestrowane.
Usunięcie węzła zabiera w DOM cały jego podrzewo, więc dla odrzuconego poddrzewa
emitujemy jedną komendę na jego korzeń, bez rekurencji.

Obsługa w JS:

```ts
// NodeAdopt
const node = this.snapshotNodes[snapshot];
this.nodes.set(id, node);
if (node instanceof Element) {
    injects(node, this.appLocation);
}

// SnapshotRemove
this.snapshotNodes[snapshot]?.remove();
```

Wywołanie `injects` jest obowiązkowe — dziś robi je `claimNode`, a bez niego
przestałoby działać przechwytywanie kliknięć w linki przez router.

### 7.3. Kierunek wiązania

**Kluczem zostaje `DomId` vertigo**, a pod niego podstawiamy węzeł przeglądarki
— nigdy odwrotnie. Dzięki temu wszystko, co przyjdzie później (`SetAttr`,
`UpdateText`, `CallbackAdd`, patche z subskrypcji reaktywnych), trafia
w niezmienione identyfikatory i nie wymaga żadnej wiedzy o hydracji.

Węzły `html`, `head` i `body` nie są adoptowane, bo `MapNodes` rozwiązuje id 1,
2 i 3 dynamicznie.

## 8. Indeks drzewa docelowego

`HashMap<DomId, TargetNode>` budowany z bufora komend, tak jak dzisiejsze
`createVirtualNodes`, tylko na typowanym `Vec<DriverDomCommand>` zamiast na
obiektach zdekodowanych z bajtów.

```rust
struct TargetNode {
    kind: TargetKind,
    attrs: BTreeMap<StaticString, String>,
    children: Vec<DomId>,
}

enum TargetKind {
    Element { name: StaticString },
    Text { value: String },
    Comment { value: String },
}
```

Reguły odtwarzania:

- `CreateNode` / `CreateText` / `CreateComment` — zakładają węzeł.
- `UpdateText` **nadpisuje** wartość z `CreateText`. `CreateText` bywa
  nieaktualne: `Computed` odczytany przy otwartej transakcji zwraca wartość
  z cache, więc `DomText::patched` może zapiec starą treść i poprawić ją
  natychmiast po.
- `SetAttr` / `RemoveAttr` — modyfikują mapę atrybutów.
- `InsertBefore` — **najpierw odczepia dziecko od dotychczasowego rodzica**,
  potem wstawia przed `ref_id` albo na koniec. Bez odczepienia węzeł przeniesiony
  między rodzicami pojawia się w dwóch miejscach. Serwer robi to samo
  (`AllElements::insert_before` woła `remove_from_parent`).
- `RemoveNode` / `RemoveText` / `RemoveComment` — usuwają węzeł i odczepiają go
  od rodzica.

Węzły utworzone i usunięte w obrębie jednego mountu znikają. Węzły nigdy nie
wstawione są sierotami i nie biorą udziału w dopasowaniu.

Indeks żyje tylko przez czas `flush_hydration()` i jest po nim porzucany.

## 9. Dopasowanie

Rekurencyjne, startuje z dwóch par: `body` (id 3) ze `snapshot.body` i `head`
(id 2) ze `snapshot.head`. Jeśli snapshot nie ma `body`, hydracja nie ma od
czego zacząć — patrz 11.3.

Atrybuty `html`, `head` i `body` uzgadniamy tak jak dla węzłów adoptowanych,
mimo że same nie są adoptowane.

Dla każdego dziecka docelowego, w kolejności, przesuwając kursor po dzieciach
snapshotu:

- **Element** — szukamy od kursora pierwszego elementu o zgodnej nazwie.
  Wszystko pominięte po drodze trafia na listę do usunięcia. Po trafieniu:
  `NodeAdopt`, uzgodnienie atrybutów w obie strony, rekurencja w głąb, kursor
  za trafienie.
- **Tekst** — patrz 9.3.
- **Komentarz** — patrz 9.4.

Po wyczerpaniu dzieci docelowych wszystkie pozostałe dzieci snapshotu trafiają
na listę do usunięcia.

**Reguła przy braku dopasowania.** Jeśli dla dziecka docelowego nie znajdzie się
odpowiednik, tworzymy je od nowa i **nie przesuwamy kursora** ani nic nie
usuwamy. Pomijanie i usuwanie węzłów snapshotu zachodzi wyłącznie wtedy, gdy
dopasowanie faktycznie znaleziono dalej w liście. Ta asymetria jest celowa:
niedopasowane dziecko docelowe nie jest dowodem, że zawartość snapshotu jest
śmieciem. Odpowiada to dzisiejszej semantyce `removeSkippedNodes`, które
wywoływane jest tylko w gałęzi trafienia.

Uzgodnienie atrybutów działa w obie strony: atrybuty obecne w snapshocie
a nieobecne w węźle docelowym są usuwane (`RemoveAttr`), różniące się lub
brakujące są ustawiane (`SetAttr`). Serwer mógł wyrenderować atrybut, którego
drzewo klienckie nie ma — renderuje z tego samego strumienia, ale komponent
może pod `is_browser()` narysować coś innego, i wtedy zostawiony `href` albo
`disabled` przeżyłby na węźle, który należy już do przeglądarki.

### 9.1. Nazwy tagów

JS wysyła `tagName` zmniejszone do małych liter. Rust porównuje przez
`eq_ignore_ascii_case` ze swoją nazwą po zdjęciu opcjonalnego prefiksu `svg:`.

Dzięki temu nie przenosimy do wasma zbioru `SVG_TAGS` (60 nazw) ani funkcji
`expectedTagName`. Dziś są one potrzebne, bo elementy HTML raportują `tagName`
wielkimi literami, a SVG zachowują swoją wielkość liter; porównanie bez
uwzględniania wielkości liter jest poprawne dla obu światów naraz. `tags.ts`
zostaje w JS, gdzie i tak musi zostać — `createElement` potrzebuje zbioru do
wyboru przestrzeni nazw.

Koszt: porównanie jest teoretycznie bardziej pobłażliwe niż dzisiejsze, bo
`svg:a` dopasowałoby się do HTML-owego `<a>`. W praktyce jest to nieosiągalne
bez wcześniejszego dopasowania rodzica, czyli `<svg>`.

### 9.2. Białe znaki z formatowania

Szukając dopasowania dla **elementu**, wolno pominąć i usunąć węzeł tekstowy
snapshotu zbudowany z samych białych znaków.

Jest to bezpieczne, bo konteksty się nie nachodzą: tam, gdzie białe znaki są
znaczące (`<pre>`, treść inline), `convert_to_string` przechodzi na
`Format::none()` i nie wstrzykuje niczego — a wtedy odpowiedni węzeł tekstowy
istnieje również w drzewie docelowym i zostaje skonsumowany zwykłą gałęzią
tekstową, zanim dojdzie do szukania elementu.

### 9.3. Scalone węzły tekstowe

Kilka sąsiadujących `DomText` serwer zlepia w jeden przebieg tekstowy
(`last_text_add` w `get_render_child_mode`), a parser robi z tego **jeden**
węzeł `Text`.

Dziś JS przypisuje ten węzeł pierwszemu z nich, a pozostałe **liczy jako
dopasowane, nie wiążąc ich z niczym** (`skipTextVNodes` w `hydration.ts`).
Skutek: późniejszy `UpdateText` na drugim z nich trafia w id nieobecne
w `MapNodes` i rzuca błąd.

Nowa reguła: scalony węzeł adoptuje **pierwszy** tekst docelowy i dostaje
`UpdateText` obcinający go do własnej wartości; rodzeństwo powstaje od nowa
przez `CreateText` plus `InsertBefore`. Jeden węzeł reużyty, wszystkie id
powiązane, żadnego wiszącego identyfikatora.

**Rozpoznanie przypadku scalonego** odbywa się po stronie drzewa docelowego, nie
przez analizę treści snapshotu: liczymy ciąg następujących po sobie dzieci
tekstowych węzła docelowego. Jest to wystarczające, bo serwer zlepia dokładnie
takie ciągi. Gdy ciąg ma długość 1, porównujemy treść i emitujemy `UpdateText`
tylko wtedy, gdy się różni. Gdy ma długość większą niż 1, pierwszy adoptuje
i zawsze dostaje `UpdateText`, a pozostałe powstają od nowa.

### 9.4. Komentarze-markery

Markery `render_value`/`render_list` nie mają w HTML odpowiednika, bo SSR
wycina komentarze. Zawsze tworzymy je od nowa i **nie przesuwamy** kursora
snapshotu.

### 9.5. Elementy infrastrukturalne w `<head>`

`<style>` z pakietem CSS, wstrzyknięty przez SSR, nie jest węzłem vertigo —
CSS jedzie osobnym kanałem (`InsertCss`), a JS trzyma własny element `style`
w `MapNodes` i dokleja go przez `addStyles()`. Element serwerowy zostanie więc
resztką i pójdzie do usunięcia, a JS doda swój. Jest to zachowanie identyczne
z dzisiejszym (`removeInitNodes` robi to samo), więc nie pogarszamy ryzyka
mignięcia niestylowanej treści, ale też go nie naprawiamy — to osobny temat.

## 10. Emisja strumienia

Kolejność:

1. `NodeAdopt` — wiązania.
2. `CreateNode` / `CreateText` / `CreateComment` — węzły bez dopasowania.
3. `SetAttr` / `RemoveAttr` / `UpdateText` — tylko różnice.
4. `InsertBefore` — **tylko tam, gdzie pozycja się różni**. Węzeł adoptowany,
   który już stoi na swoim miejscu, nie generuje niczego.
5. Grupa przechodząca: `InsertCss`, `CallbackAdd`, `CallbackRemove`.
6. `SnapshotRemove` — resztki.

Usunięcia `RemoveNode`/`RemoveText`/`RemoveComment` i tak wędrują na koniec
przez istniejące `sort_commands`; `SnapshotRemove` dołącza do tej grupy.

Stąd bierze się skurczenie startowej paczki: zamiast „stwórz wszystko
i powstawiaj" leci „przejmij i popraw różnice".

## 11. Polityki

Jeden kanał obsługuje trzy sytuacje, bez żadnej polityki w JS.

### 11.1. Przeglądarka, hydracja włączona

JS zwraca snapshot, Rust dopasowuje zgodnie z 9 i 10.

### 11.2. Przeglądarka, `--disable-hydration`

JS zwraca snapshot identycznie. Rust czyta flagę przez istniejące `get_env`
(CLI wstawia ją jako `data-env-disable-hydration`, tak jak pozostałe zmienne
środowiskowe), nie adoptuje niczego i emituje `SnapshotRemove` dla wszystkich
dzieci `head` i `body` oraz bufor komend bez zmian. Dziś odpowiada temu
`removeInitNodes()`; decyzja przenosi się do Rusta, mechanizm zostaje ten sam.

### 11.3. Brak snapshotu

Host SSR odpowiada `JsJson::Null` (nie ma DOM do zwrócenia). Rust wysyła bufor
bez zmian. Jedyna różnica dla `vertigo-cli` jest taka, że dostanie jedną paczkę
zamiast dwóch, co dla `AllElements` jest obojętne, bo tylko odtwarza strumień.

Nowy wariant `CommandForBrowser` wymusza dopisanie gałęzi w `handle_command`
w `crates/vertigo-cli/src/serve/server_state.rs` — kompilator tego dopilnuje,
bo `match` jest wyczerpujący.

Ta sama ścieżka obsługuje snapshot bez `body`: logujemy błąd (tak jak dziś robi
to `hydration.ts`) i wysyłamy bufor bez zmian.

## 12. Diagnostyka

Raport liczy Rust i publikuje przez istniejący kanał `JsApiCall`:

```rust
get_driver()
    .dom_access()
    .root("window")
    .set("__vertigo_hydration", report.to_json())
    .exec();
```

`DomAccess` ma już `root` i `set`, więc nie potrzeba nowej komendy ani żadnej
wiedzy o hydracji w JS. Zachowujemy dzisiejsze pola (`rootFound`, `matched`,
`hydratable`, `skipped`, `total`), bo `tests/demo/ssr.rs` je czyta — zmienia się
tylko źródło. Podsumowanie tekstowe idzie przez zwykły `log`.

## 13. Co znika z JS

- `hydration.ts` — całość, 373 linie.
- `hydration.test.ts` — zastąpiony testami w Ruście.
- `MapNodes`: `initNodes`, `removeInitNodes`, `hasInitNodes`, `claimNode` —
  cała maszyneria zapamiętywania i sprzątania węzłów SSR.
- `dom.ts`: gałąź pierwszej paczki oraz strażnicy `if (this.nodes.has(id)) return`
  w `createNode`/`createText` — Rust przestaje emitować tworzenie dla węzłów
  adoptowanych, więc są martwym kodem.
- `dom_wire.ts`: obiektowy dekoder `decodeCommands` i typ `CommandType` wychodzą
  ze ścieżki wykonania (gorąca ścieżka aplikuje wprost z kursora; dekoder
  istniał wyłącznie dla hydracji). Zostają jako narzędzie testowe dla
  `dom_wire.test.ts`, gdzie nie wchodzą do bundla.

Dochodzi: obsługa `NodeAdopt` i `SnapshotRemove` (7.2), budowa snapshotu
(6, ok. 30 linii przejścia po DOM) oraz cykl życia `snapshotNodes` (6.3).

## 14. Testy

**Nowe, jednostkowe w Ruście.** Dajemy syntetyczny snapshot i bufor komend,
sprawdzamy wyemitowany strumień. Przenosimy przypadki z `hydration.test.ts`
i dokładamy te, których dziś nie ma:

- rozcięcie scalonego przebiegu tekstowego (9.3),
- pomijanie białych znaków z formatowania (9.2),
- wielkość liter w SVG (9.1),
- uzgadnianie atrybutów w obie strony (9),
- usuwanie resztek, w tym całych poddrzew jedną komendą (7.2),
- markery komentarzy bez przesuwania kursora (9.4),
- `InsertBefore` pomijane dla węzłów stojących na miejscu (10),
- polityka `--disable-hydration` (11.2),
- brak snapshotu i snapshot bez `body` (11.3).

**Rozszerzane.** Międzyjęzykowy fixture formatu druciarskiego o tagi 14 i 15
(`command_wire.rs` plus `dom_wire.test.ts`).

**Aktualizowane, bo mierzą dokładnie to, co zmieniamy.**

- `crates/vertigo/src/tests/mount_batching.rs` — nadal jedna paczka, ale o innej
  treści.
- `crates/vertigo/src/tests/dom_command_counts.rs` — liczby komend przy mount.
- `tests/demo/ssr.rs` — źródło raportu (12); asercje pokrycia powinny wyjść
  nie gorzej niż dziś.

## 15. Odrzucone alternatywy

### 15.1. Przechowywanie atrybutów i tekstów w drzewie `DomNode`

Semantycznie najczystsze: `DomElement` dostaje mapę atrybutów, `DomText` swoją
treść, i chodzimy po prawdziwym drzewie zamiast odtwarzać je ze strumienia.

Odrzucone, bo to stały koszt pamięci i rozmiaru wasma w **każdej** aplikacji za
funkcję wykonywaną raz na start, plus druga kopia prawdy do pilnowania przy
reaktywnych atrybutach. Kod idzie konsekwentnie w przeciwnym kierunku:
`ClassState` siedzi inline, żeby nie alokować, `attr_static` istnieje po to, by
nie wciągać generyków, dekodowanie fetch cache jest odroczone, żeby linker mógł
je wyrzucić.

### 15.2. Dokładne identyfikatory z SSR (`data-v-id`)

Serwer emituje id węzłów do HTML, snapshot je przynosi, dopasowanie jest
lookupem zamiast heurystyki.

Odrzucone, bo `DomId` pochodzi z globalnego, inkrementalnego licznika: każda
rozbieżność ścieżki kodu między serwerem a przeglądarką — choćby `is_browser()`,
które vertigo jawnie wspiera i pokazuje w dokumentacji — przesuwa wszystkie
kolejne id i daje cichy rozjazd. Do tego puchnie HTML i wycieka wewnętrzny numer
do publicznego wyjścia.

Do rozważenia kiedyś jako **wzmocnienie** tego projektu: id jako podpowiedź
z fallbackiem strukturalnym.

### 15.3. Czekanie na operacje asynchroniczne

Uzasadnienie odrzucenia w 2.1.

## 16. Kryteria sukcesu

1. Pokrycie hydracji na trasach sprawdzanych przez `tests/demo/ssr.rs` nie
   gorsze niż przed zmianą.
2. Startowa paczka `DomBulkUpdate` mniejsza niż przed zmianą dla strony
   renderowanej serwerowo — zamiast tworzenia wszystkich węzłów zawiera
   adopcje i różnice.
3. Żadna decyzja dopasowania nie jest podejmowana w JS. Kod JS zna tylko:
   „zbuduj snapshot", „zwiąż id z węzłem o indeksie", „usuń węzeł o indeksie".
4. Algorytm dopasowania jest w pełni testowalny bez przeglądarki.
5. Interaktywność nie jest odroczona względem stanu przed zmianą — nadal jedna
   paczka na koniec mountu, z callbackami w środku.
6. `--disable-hydration` zachowuje dzisiejsze zachowanie widoczne z zewnątrz.
