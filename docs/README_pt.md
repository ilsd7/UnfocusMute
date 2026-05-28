<div align="center">
  <img src="../assets/app-icon.png" alt="Ícone do UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>App leve para a bandeja do Windows que silencia automaticamente jogos e apps selecionados quando perdem o foco.</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · <a href="README_fr.md">Français</a> · Português · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Totalmente local &nbsp;·&nbsp; Sem acesso à rede &nbsp;·&nbsp; Sem telemetria &nbsp;·&nbsp; Não requer instalação</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Baixar</a>
    · <a href="#uso">Uso</a>
    · <a href="#segurança-e-privacidade">Privacidade</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute é um app pequeno e leve para a bandeja do Windows que silencia automaticamente jogos e apps selecionados quando passam para segundo plano e restaura o áudio quando voltam ao primeiro plano.

- Criado como app nativo em Rust, ele roda sem runtime separado.
- O executável tem cerca de 500 KB.
- Ele não se limita a jogos: você também pode registrar apps comuns, como navegadores, mensageiros, launchers e players de mídia.
- O silenciamento e a restauração se aplicam apenas às sessões que o UnfocusMute alterou diretamente; sessões que você já tinha silenciado não são modificadas.

<p align="center">
  <img src="../assets/screenshot_pt.png" width="600" alt="Janela principal do UnfocusMute">
</p>

---

## Útil quando

- Você costuma usar Alt+Tab para sair de um jogo ou app enquanto ele continua aberto.
- Você quer manter silencioso um app que não tem opção de silenciar em segundo plano.
- Você quer silenciar apenas o áudio em segundo plano de um app específico enquanto faz outras coisas.

## Recursos

- Silencia automaticamente apps registrados quando passam para segundo plano e restaura o áudio quando voltam ao primeiro plano.
- Permite escolher um app com sessão de áudio, encontrá-lo em `Todos os processos` ou digitar um nome como `game.exe`.
- Registra o app inteiro por `.exe`, ou apenas a instância em execução por PID.
- Notas por app, status em tempo real e pausa / retomada individual.
- Continua monitorando pela bandeja depois que a janela é fechada, com ações para abrir, ocultar, pausar tudo e sair.
- Opções para `Iniciar minimizado na bandeja`, `Início automático ao entrar no Windows` e `Ao sair, restaurar o áudio silenciado pelo UnfocusMute`.
- Escolha de idioma na primeira execução e troca imediata entre 9 idiomas dentro do app.

---

## Baixar e executar

No Windows 10/11, baixe o pacote ZIP, extraia e execute o app.

| Pacote mais recente |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Arquivo SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notas da versão](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Depois de extrair o ZIP, mova a pasta `UnfocusMute-windows-x64` para o local onde você quer manter o app e execute `UnfocusMute-v<version>.exe` dentro dela.

UnfocusMute é um app independente que não requer instalação. Você também não precisa instalar Rust, Visual Studio Build Tools, MinGW nem outras ferramentas de desenvolvimento.

> **Observação:** Como certificados de assinatura de código têm custo, o app atualmente é distribuído sem assinatura de código do Windows. No primeiro uso, podem aparecer avisos do Windows SmartScreen ou de "editor desconhecido". Se quiser verificar a integridade do arquivo por conta própria, consulte [Verificar arquivos da versão](#verificar-arquivos-da-versão).

## Antes de usar

UnfocusMute funciona com base nos nomes de processo, nas informações da janela em primeiro plano e nas sessões CoreAudio fornecidas pelo Windows.

Por isso, se um driver, uma permissão ou uma ferramenta de segurança limitar o acesso à sessão, o controle de silenciamento pode ficar parcialmente limitado.

A lista padrão mostra apenas apps que já têm uma sessão de áudio naquele momento. Se o app ainda não criou uma sessão de áudio, alterne para `Todos os processos` para procurá-lo entre os processos `.exe` em execução. Use `Apenas sessões de áudio` para voltar à lista filtrada.

**Comportamento ao registrar por PID:** O Windows nem sempre informa o mesmo PID para uma sessão de áudio e para a janela em primeiro plano. Para compensar isso, o UnfocusMute trata o app como tendo voltado ao primeiro plano quando o nome `.exe` do PID registrado corresponde ao nome `.exe` da janela atual em primeiro plano.

Assim, se houver várias instâncias do mesmo `.exe` em execução ao mesmo tempo, talvez não seja possível distinguir perfeitamente um PID específico. Nesse caso, o som pode ser restaurado quando outra instância estiver em foco.

**Compatibilidade com anti-cheat:** UnfocusMute não injeta código em jogos, não lê memória do jogo, não intercepta entrada e não modifica arquivos do jogo. Ele usa apenas informações de processo/janela em primeiro plano do Windows e controles de silenciamento de sessões CoreAudio, então espera-se que funcione sem problemas na maioria dos anti-cheats, mas não é possível garantir compatibilidade com todos eles.

## Uso

1. Abra o UnfocusMute.
2. Escolha o idioma na tela exibida na primeira execução. O idioma padrão é inglês.
3. Inicie o jogo ou app que você quer silenciar quando estiver em segundo plano.
4. Escolha um app na lista ou use `Pesquisar processo` para buscar pelo nome, depois clique em `Registrar`. Se o app ainda não criou uma sessão de áudio, alterne para `Todos os processos` para consultar a lista de todos os processos em execução. Use `Apenas sessões de áudio` para voltar à lista filtrada. Se ele não estiver na lista, digite o nome `.exe` manualmente.
5. Se precisar registrar apenas um PID específico, clique em `Ver PID` e escolha a entrada individual. Entradas por PID valem só para a instância em execução; se o app reiniciar com outro PID, selecione novamente.
6. Clique com o botão direito em um app registrado para editar sua nota ou usar `Pausar`.
7. Abra `Configurações` no canto inferior esquerdo para alterar opções de comportamento.
8. Ao fechar a janela, o app continua rodando na bandeja e monitorando os apps registrados. Use `Sair` para encerrar completamente.

## Usar notas em apps registrados

Se o nome do processo não for suficiente para lembrar qual app é, clique com o botão direito no app registrado e escolha `Editar nota`. A nota aparece acima do nome do processo na lista de apps registrados e não afeta a regra de detecção.

Isso ajuda quando um mesmo launcher de jogo abre vários processos ou quando o nome do executável não deixa claro para que ele serve.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - cliente do servidor de teste`

As notas são salvas localmente junto com o restante das configurações em `%APPDATA%\UnfocusMute\config.json`.

## Encontrar o nome do executável

Se você não souber qual nome registrar, confira no Gerenciador de Tarefas o executável que termina em `.exe`.

1. Inicie primeiro o app que você quer registrar.
2. Use `Alt`+`Tab` ou `Windows`+`Tab` para sair da tela do jogo e voltar ao Windows.
3. Pressione `Ctrl`+`Shift`+`Esc` para abrir o Gerenciador de Tarefas.
4. Ordene a lista de processos por `CPU` e encontre o app que acabou de iniciar.
5. Clique com o botão direito nesse item e abra `Propriedades`.
6. Veja o nome do executável que termina em `.exe`, como `game.exe`, e adicione-o ao UnfocusMute.

## Solução de problemas

Se um app não aparecer na lista, ou se o registro por PID não funcionar como esperado, confira primeiro [Antes de usar](#antes-de-usar) e [Encontrar o nome do executável](#encontrar-o-nome-do-executável).

Se o status no topo mudar para `Atenção`, clique em `Detalhes` para ver a mensagem de erro detalhada.

Se o problema continuar, informe no GitHub Issues.

Se você suspeitar de uma vulnerabilidade de segurança, não publique detalhes em uma Issue pública. Use o processo de relato privado e consulte [SECURITY.md](../SECURITY.md) para mais informações.

## Arquivo de configuração

Se precisar verificar ou fazer backup do arquivo de configuração diretamente, clique em `Abrir pasta de configuração` em Configurações.

O Explorador de Arquivos abre a pasta `%APPDATA%\UnfocusMute`, onde as configurações são salvas.

Você pode editar o arquivo de configuração diretamente, mas se o formato estiver inválido e não puder ser lido, ele será salvo como backup em `config.invalid-<timestamp>.json`. Se o problema for encontrado durante a inicialização do app, as configurações serão restauradas para os padrões; se for encontrado enquanto o app estiver em execução, um novo arquivo de configuração será criado com base nas configurações atuais do app.

---

## Segurança e privacidade

UnfocusMute roda totalmente de forma local. Ele funciona normalmente mesmo sem conexão com a internet e não exige permissões de administrador. Também não faz solicitações de rede automáticas, não usa telemetria, não envia relatórios de falha, não envia logs remotos nem coleta dados.

Como exceção, o repositório GitHub deste projeto só é aberto no navegador padrão quando você clica no botão `Repositório GitHub` em Configurações.

A detecção de sessões de áudio e o controle de silenciamento usam apenas as APIs CoreAudio do Windows. O UnfocusMute não injeta código nos processos de destino, não lê a memória deles nem intercepta entrada do usuário.

### Informações salvas

O UnfocusMute salva apenas as configurações necessárias para funcionar em `%APPDATA%\UnfocusMute\config.json`.

- Nomes de processos registrados
- PIDs registrados diretamente
- Último estado de silenciamento dos apps registrados
- Notas que você escrever
- Idioma e configurações escolhidos
- Posição da janela

Essas informações não são enviadas para nenhum lugar.

Se você ativar o início automático ao entrar no Windows, o caminho do executável atual também será salvo no valor `UnfocusMute` em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Informações que não são salvas

O UnfocusMute não salva histórico de uso, logs de atividade, logs de erro, dados de áudio, títulos de janelas, teclas digitadas nem qualquer informação que não esteja listada acima em "Informações salvas".

### Como remover

Para remover todos os arquivos relacionados ao app, apague a pasta `UnfocusMute-windows-x64` e depois apague `%APPDATA%\UnfocusMute`.

Se você já ativou o início automático, apague também o valor `UnfocusMute` em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

---

## Verificar arquivos da versão

Não presuma que os arquivos enviados ao GitHub Releases sempre correspondem ao código-fonte publicado no repositório.

Se permissões de publicação forem abusadas ou uma conta for comprometida, arquivos criados a partir de outro código ou arquivos modificados podem ser enviados para uma versão publicada.

Por transparência, o UnfocusMute oferece uma forma de verificar se os arquivos enviados ao GitHub Releases são artefatos oficiais gerados pelo GitHub Actions a partir do código-fonte da tag correspondente neste repositório.

O ZIP da versão e o arquivo de verificação SHA-256 são gerados automaticamente pelo GitHub Actions, e cada arquivo é fornecido com uma atestação de procedência da compilação (attestation).

Os comandos abaixo permitem confirmar que o ZIP baixado foi gerado pela compilação oficial deste repositório.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

---

## Compilar a partir do código-fonte

O alvo recomendado para as versões publicadas é `x86_64-pc-windows-msvc`.

Requisitos:

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

As compilações das versões publicadas são configuradas para reduzir o tamanho do binário. O perfil de release em `Cargo.toml` remove símbolos, ativa LTO, usa uma única codegen unit, define `panic = "abort"` e otimiza para tamanho.

Executável:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ZIP de distribuição:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Atualizar avisos de licenças de terceiros:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

O resultado é criado em `dist\UnfocusMute-windows-x64.zip`, com um arquivo de verificação SHA-256 em `dist\UnfocusMute-windows-x64.zip.sha256`. O ZIP inclui o executável com versão (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` e os README `.txt` localizados por idioma em `docs`.

---

## Licença

Apache License 2.0. Veja [LICENSE](../LICENSE) para mais detalhes.

Veja [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) para os avisos de licenças de crates Rust de terceiros.
