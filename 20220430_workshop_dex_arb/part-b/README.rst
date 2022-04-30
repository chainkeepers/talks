Část B - Posílání arbitrážních transakcí
========================================

Z čeho se skládá zisková arbitráž
+++++++++++++++++++++++++++++++++

- série tradů tak, ze výsledná pozice je 0 všude, až na jeden asset, kde zůstane zisk
- najdu příležitost k ziskové arbitráži
- v okamžiku nalezení poslu transakci na kontrakt, který sérii swapů udělá

Arbitrážovací solidity contract
+++++++++++++++++++++++++++++++

- vytvoření obslužného kódu pomocí `hardhat`
- napsat funkci, jež umožní provést obě směny v 1 transakci
- začínáme s co nejjednodušší funkčností


Jak se zorientovat v dokumentaci DEXů a Solidity
++++++++++++++++++++++++++++++++++++++++++++++++

- Google "<DEX name> solidity" např. https://www.google.com/search?q=uniswap+v3+solidity 
- Router contract <> Pair contract
- Pozorumění sample kódu, druhy proměnných, globální proměnné


provedení arbitráže na testnetu
+++++++++++++++++++++++++++++++

- nastavení připojení na uzel testovací sítě
- nalezení a výměna adres DEXů
- získání test-ETH
- nasazení našeho kontraktu
- vyvolání směn v našem kontraktu

Navíc
=====

- https://www.chainkeepers.io/cs/hiring/
