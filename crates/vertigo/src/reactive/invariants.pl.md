# Graf reaktywny — niezmienniki

Jeden `Graph` trzyma węzły `Value`, `Computed` i subskrypcje.
Węzły z różnych grafów nie widzą się nawzajem.

**Fala** to jedno odpalenie `propagate`. **Transakcja** zbiera zapisy.
**Cutoff** znaczy: pomijamy dzieci, gdy wartość się nie zmieniła.
**Connect** / **disconnect** włączają i wyłączają pracę na zewnątrz (`when_connect`).

## Jak przebiega zapis

1. `set` albo `transaction` zapisuje wartości i oznacza je jako brudne.
2. Najbardziej zewnętrzna transakcja startuje falę.
3. Fala odświeża gotowe węzły. Niezmienione węzły nie brudzą dzieci.
4. Potem hooki `on_after_transaction`.
5. Po tym: connect i disconnect. Zakładanie i zdejmowanie handlerów na zewnątrz
   nie jest częścią fali.

`set` z `when_connect` (`create`) odpala to od nowa.
`set` z compute, subscribe albo destruktora `DropResource` leci na konsolę jako błąd
i jest ignorowany. Nic nie zapisuje.

## Niezmienniki

### 1. Compute, subscribe i Drop nie mogą pisać

Nie wołaj `Value::set` (ani `change`) z:

- funkcji compute
- callbacku subscribe
- destruktora `DropResource`
- fali, która już trwa

Compute tylko czyta. Subscribe tylko gada ze światem (DOM, logi).
Drop tylko zdejmuje zewnętrzną subskrypcję (timer, socket, `popstate`).
Żadne z nich nie może zapisać z powrotem do grafu.

Jeśli taka próba się odbędzie, zapis jest ignorowany i na konsolę leci:

```text
vertigo: Value::set is not allowed from a computed, a subscribe callback, a DropResource, or during propagation
```

Dotyczy to wszystkiego, co leci *wewnątrz* tych domknięć, nie tylko kodu napisanego w nich
wprost. Jeśli w callbacku subscribe coś zostanie zdropowane — komponent znika, bo widok jest
przebudowywany — jego `Drop` leci wewnątrz callbacku, więc `set` stamtąd też jest ignorowany.

Pisać wolno z handlerów click/input, timerów, fetcha, socketów,
`on_after_transaction` oraz `when_connect` / `Value::with_connect` (tylko `create`).

### 2. Connect i disconnect czekają na koniec fali

`when_connect` nie leci w chwili, gdy węzeł dostaje dziecko.
Disconnect nie leci w chwili, gdy traci ostatnie dziecko.
Oba czekają, aż fala się skończy i aż odpalą się hooki `on_after_transaction`.
Gdy graf nic nie liczy, lecą od razu.

Jeśli węzeł w jednej fali był obserwowany i przestał być — nic się nie dzieje.
Jeśli nie był, a potem zaczął być — connect leci raz.

`create` leci, gdy graf już spoczął, więc `Value::with_connect` może wołać `set`
(na przykład żeby wyzerować wartość przy podłączeniu). Ten `set` to nowa transakcja
i nowa fala.

Ta fala może zmienić to, kto jest obserwowany — także węzeł, który właśnie się łączy.
Po powrocie z domknięcia stan połączeń jest ponownie dopasowywany do grafu.
Węzeł, który sam siebie przestał obserwować, dostaje disconnect. Nie zostaje połączony.

Disconnect nie może pisać. Nie może z powrotem oglądać węzła, więc connect i disconnect
nie odbijają się w kółko.

`create` już może, a zapis stamtąd może sprawić, że obserwowany zaczyna być inny węzeł.
Tak jeden connect pociąga następny — i tak samo pierścień connectów mógłby podawać sobie
połączenie bez końca, gdzie każdy zapis cofa poprzedni. Dlatego flush to jedna runda
decyzji connect na węzeł: węzeł łączy się w nim najwyżej raz. Ten, który miałby połączyć
się drugi raz, zostaje rozłączony, a na konsolę leci:

```text
vertigo: when_connect closures are undoing each other - a node cannot connect twice in one flush, so it is left disconnected
```

Łańcuch — connect, który zaczyna obserwować kolejny węzeł — łączy każdy swój węzeł raz,
choćby był długi, i nigdy nie jest ucinany.

### 3. Falę startuje tylko zewnętrzna transakcja

`Value::set` sam jest transakcją.
`transaction` wewnątrz innej `transaction` tylko zapisuje i oznacza brudne.
Fala startuje, gdy wraca zewnętrzne wywołanie.

### 4. Niezmieniona wartość zatrzymuje aktualizację

Po odświeżeniu dzieci lecą tylko wtedy, gdy nowa wartość jest inna (`PartialEq`).
Dlatego `Computed<T>` wymaga `T: PartialEq`.

Subskrypcja nie ma dzieci. Nie przekazuje zmiany dalej.

### 5. Zależności biorą się z `get`, nie z deklaracji

Wywołanie `get` zapisuje ten węzeł jako rodzica tego, który właśnie się liczy.
Następne odpalenie podmienia całą listę rodziców.
Jeśli computed przestaje czytać węzeł, przestaje od niego zależeć.

Dziecko trzyma rodziców przy życiu (silne referencje).
Graf nie (słabe referencje).
Upuść ostatni uchwyt węzła — węzeł znika.

### 6. W jednej fali węzeł odświeża się najwyżej raz

Brudny węzeł jest gotowy, gdy żaden z jego rodziców nie jest już brudny.
Dzieci są oznaczane jako brudne tylko wtedy, gdy wartość rodzica się zmieniła.
Jeśli compute czyta rodzica, który jeszcze jest nieaktualny, ten rodzic jest odświeżany najpierw.
Jeśli zostają brudne węzły i żaden nie jest gotowy, albo węzeł jest odświeżany w trakcie
własnego odświeżania — jest cykl, program panikuje.

W jednej fali węzeł nie odświeża się drugi raz.

### 7. Po fali każda wartość jest poprawna

Gdy fala się kończy, każdy `Value` i `Computed` zgadza się z bieżącymi źródłami.
Subskrybent widzi jedną wartość z tej fali: tę, która zgadza się ze źródłami.

Fala trwa, aż nic nie jest brudne.
Cykl panikuje. Węzłów nie odrzucamy, żeby „uratować” falę.

### 8. Grafy się nie mieszają

`Value::new` i `Computed::from` używają jednego grafu na wątek.
`Graph::new()` robi osobny graf.
Zapis w grafie A nigdy nie widzi węzłów grafu B.
