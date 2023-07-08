# ICFPC2023

Private repo for participating in icfpc 2023

## Links

https://www.icfpcontest.com/dashboard
https://github.com/SlNPacifist/icfpc2023

## Setup

Для отправки решений скриптом:
* скопируй токен из tg-канала в файл token
* установи [jq](https://jqlang.github.io/jq/)
  debian:
  ```
  sudo apt update
  sudo apt install jq
  ```

  macos:
  ```
  brew update
  brew install jq
  ```

## Submissions

Все отправки здесь: https://www.icfpcontest.com/dashboard

```
submit.sh problem-id path-to-solution-data
submit.sh 7 ./solutions/problem-7.json
```

## Folder structure

`code/*` - код решений. Любимые языки разные, решений может быть много.
`data/*` - исходные данные
`solutions/*` - оптимальные ответы
`solutions-xxx/*` - субоптимальные ответы
