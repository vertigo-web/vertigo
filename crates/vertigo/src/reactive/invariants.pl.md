# Graf reaktywny — niezmienniki

(wersja robocza)

Jeden `Graph` trzyma węzły `Value`, `Computed` i subskrypcje.
Węzły z różnych grafów nie widzą się nawzajem.

**Fala** to jedno odpalenie `propagate`. **Transakcja** zbiera zapisy.
**Cutoff** znaczy: pomijamy dzieci, gdy wartość się nie zmieniła.
**Connect** / **disconnect** włączają i wyłączają pracę na zewnątrz (`when_connect`).

## Jak przebiega zapis

1. `set` albo `transaction` zapisuje wartości i oznacza je jako brudne.
2. Najbardziej zewnętrzna transakcja startuje falę.
3. Fala odświeża gotowe węzły. Niezmienione węzły nie brudzą dzieci.
4. Po fali: connect i disconnect.
5. Potem hooki `on_after_transaction`.

`set` z `when_connect` odpala to od nowa.
`set` z compute albo subscribe leci na konsolę jako błąd i jest ignorowany. Nic nie zapisuje.

## Niezmienniki

### 1. Compute i subscribe nie mogą pisać

Nie wołaj `Value::set` (ani `change`) z:

- funkcji compute
- callbacku subscribe
- fali, która już trwa

Compute tylko czyta. Subscribe tylko gada ze światem (DOM, logi).
Żadne z nich nie może zapisać z powrotem do grafu.

Jeśli taka próba się odbędzie, zapis jest ignorowany i na konsolę leci:

```text
vertigo: Value::set is not allowed from a computed, a subscribe callback, or during propagation
```

Dotyczy to wszystkiego, co leci *wewnątrz* tych domknięć, nie tylko kodu napisanego w nich
wprost. Jeśli w callbacku subscribe coś zostanie zdropowane — komponent znika, bo widok jest
przebudowywany — jego `Drop` leci wewnątrz callbacku, więc `set` stamtąd też jest ignorowany.
Żeby wyczyścić wartość przy odmontowaniu, zrób to z zasobu `when_connect`: te są dropowane
po fali.

Pisać wolno z handlerów click/input, timerów, fetcha, socketów,
`on_after_transaction` oraz `when_connect` / `Value::with_connect`.

### 2. Connect i disconnect czekają na koniec fali

`when_connect` nie leci w chwili, gdy węzeł dostaje dziecko.
Disconnect nie leci w chwili, gdy traci ostatnie dziecko.
Oba czekają, aż fala się skończy.
Gdy graf nic nie liczy, lecą od razu.

Jeśli węzeł w jednej fali był obserwowany i przestał być — nic się nie dzieje.
Jeśli nie był, a potem zaczął być — connect leci raz.

`when_connect` leci po fali, więc `Value::with_connect` może wołać `set`.
Ten `set` to nowa transakcja i nowa fala.

Ta fala może zmienić to, kto jest obserwowany — także węzeł, który właśnie się łączy.
Po powrocie z domknięcia stan połączeń jest ponownie dopasowywany do grafu.
Węzeł, który sam siebie przestał obserwować, dostaje disconnect. Nie zostaje połączony.

Connect i disconnect nie mogą się nawzajem cofać.
Connect, który przestaje obserwować własny węzeł, a jego disconnect zaczyna z powrotem —
to pętla bez końca.
Po 100 connectach jednego węzła w jednym flushu pętla jest ucinana:
leci błąd do logu, a węzeł zostaje rozłączony.
Łańcuch — connect, który zaczyna obserwować kolejny węzeł — to nie pętla i nigdy nie jest ucinany.

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
