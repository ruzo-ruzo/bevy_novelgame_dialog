最初の文言
==========
解説
-----------
この背景表示するために滅茶苦茶苦労してしまった……。  
本体は手前のこの文章のはずなのに。[^wait]
[jump](explain02.md#キャラ選択肢)

キャラ選択肢
===========
解説
-----------
どっちと話す？[^wait]
* [子供](explain02.md#子供選択肢)
* [ウサギ](explain02.md#ウサギ選択肢)
* [終わる](explain02.md#閉じる)

ウサギ選択肢
===========
うさぎ
-----------
選択肢ボックスを開きます。[^wait]
* [ウサギ挨拶して](explain02.md#ウサギ挨拶)
* [ウサギ拍手して](explain02.md#ウサギ拍手)
* [戻る](explain02.md#キャラ選択肢)

ウサギ挨拶
===========
ウサギが挨拶します[^wait]
[^signal(Rabit_greeting)]
[^wait]
[jump](explain02.md#ウサギ選択肢)

ウサギ拍手
===========
ウサギが拍手します[^wait]
[^signal(Rabit_clap)]
[^wait]
[jump](explain02.md#ウサギ選択肢)

子供選択肢
===========
こども
-----------
選択肢ボックスを開きます。[^wait]
* [子供挨拶して](explain02.md#子供挨拶)
* [子供拍手して](explain02.md#子供拍手)
* [戻る](explain02.md#キャラ選択肢)

子供挨拶
===========
子供が挨拶します[^wait]
[^signal(Girl_bow)]
[^wait]
[jump](explain02.md#子供選択肢)

子供拍手
===========
子供が拍手します[^wait]
[^signal(Girl_clap)]
[^wait]
[jump](explain02.md#子供選択肢)

閉じる
===========
ではこれにて。[^wait]
[^close]

[^wait]: 入力待ち  
[^signal(Rabit_greeting)]: うさぎ挨拶モーション再生  
[^signal(Rabit_clap)]: うさぎ拍手モーション再生  
[^signal(Girl_clap)]: こども挨拶モーション再生  
[^signal(Girl_bow)]: こども拍手モーション再生
[^close]: ダイアログボックスを閉じる  