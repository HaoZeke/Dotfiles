" Use Vim settings, rather then Vi settings (much better!).
" This must be first, because it changes other options as a side effect.
"set nocompatible

" ================ Plugins! ======================

if empty(glob('~/.vim/autoload/plug.vim'))
  silent !curl -fLo ~/.vim/autoload/plug.vim --create-dirs
    \ https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim
  autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
endif
" function
function! BuildYCM(info)
  " info is a dictionary with 3 fields
  " - name:   name of the plugin
  " - status: 'installed', 'updated', or 'unchanged'
  " - force:  set on PlugInstall! or PlugUpdate!
  if a:info.status == 'installed' || a:info.force
    !python2 ./install.py
  endif
endfunction
    " :PlugInstall is required. Along with a source %
    " Initialize List of plugins...
call plug#begin()
    " The basis."
    Plug 'tpope/vim-sensible'
    Plug 'vim-scripts/L9'

    Plug 'Shougo/vimproc.vim', { 'do': 'make' }
    "{ 'for': 'cpp' } WASTE. Python2 ONLY
   " Plug 'Valloric/YouCompleteMe', { 'do': function('BuildYCM') } | Plug 'l3nkz/ycmtex' 
   " autocmd! User YouCompleteMe call youcompleteme#Enable()

    " NERD tree will be loaded on the first invocation of NERDTreeToggle command
    Plug 'scrooloose/nerdtree'
    Plug 'Xuyuanp/nerdtree-git-plugin'
    " WebDev
    Plug 'mohitleo9/vim-fidget', {'for': 'clojure'  } 
    " Loaded when clojure file is opened
    Plug 'tpope/vim-fireplace', { 'for': 'clojure' }

    " Because everything must be tracked
    Plug 'wakatime/vim-wakatime'

    " Group dependencies, vim-snippets depends on ultisnips
    " Plug 'SirVer/ultisnips' | Plug 'honza/vim-snippets'
      
    " Awesome syntax checker.
    " REQUIREMENTS: See :h syntastic-intro
"  function! Installjshint(info)
"    if a:info.status == 'installed' || a:info.force
"        !sudo npm install -g jshint
"      endif
"  endfunction
"  Plug 'scrooloose/syntastic', { 'do': function('Installjshint') }


    " Markdown Auto View. VERY BAD!! BREAKS SYSTEM INTERMITTENTLY
    " REQUIREMENTS: See :h syntastic-intro
"  function! Installinstamark(info)
"   if a:info.status == 'installed' || a:info.force
"        !sudo npm install -g instant-markdown-d
"      endif
"  endfunction
"  Plug 'suan/vim-instant-markdown', { 'do': function('Installinstamark') }

    " Group dependencies, Pandoc doesn't depend but should.. Who uses this!?
   " Plug 'vim-pandoc/vim-pandoc' | Plug 'vim-pandoc/vim-pandoc-syntax'
      
    " Easy motions.
    Plug 'Lokaltog/vim-easymotion'

    " Super easy commenting, toggle comments etc
    Plug 'scrooloose/nerdcommenter'

    " Autoclose (, " etc
    Plug 'Townk/vim-autoclose'

    " Git wrapper inside Vim
    Plug 'tpope/vim-fugitive'

    " Handle surround chars like ''
    Plug 'tpope/vim-surround'

    " Align your = etc.
    Plug 'vim-scripts/Align'

    " Better Minibuff Kinda Pointless?
    " Plug 'weynhamz/vim-plugin-minibufexpl'

    " Javascript
    "Plug 'pangloss/vim-javascript'
    Plug 'othree/javascript-libraries-syntax.vim'
    Plug 'othree/yajs.vim'

    " Julia
    Plug 'JuliaLang/julia-vim'
    " Css
    Plug 'skammer/vim-css-color'

    " Less
    Plug 'groenewege/vim-less'

    " I need Colemak
    Plug 'jooize/vim-colemak'

    " Python
    Plug 'klen/python-mode'
    Plug 'jmcantrell/vim-virtualenv'  " This should fix the virtualenvs.

    " Movement
    Plug 'matze/vim-move'
    
    " Git Stuff
    Plug 'airblade/vim-gitgutter'
    Plug 'mattn/gist-vim'
    Plug 'gregsexton/gitv'
    " Latex!!!! Latex Box/atp_vim is obsoleted by vimtex.
    " Plug 'vim-latex/vim-latex'
    " Plug 'gerw/vim-latex-suite'
     "Plug 'coot/atp_vim'
    "Plug 'LaTeX-Box-Team/LaTeX-Box'
    Plug 'gi1242/vim-tex-syntax'
    Plug 'lervag/vimtex'

    " Not that powerline isn't perfect... right?
    Plug 'bling/vim-airline'
    Plug 'vim-airline/vim-airline-themes'
    " Undo tree
    Plug 'vim-scripts/Gundo'

    " Apparently, gvim in vim
    Plug 'vim-scripts/CSApprox'

    " Why we theme
    Plug 'tomasr/molokai'
    Plug 'flazz/vim-colorschemes'
    " File searching with faster matching
    Plug 'ctrlpvim/ctrlp.vim'
    Plug 'FelikZ/ctrlp-py-matcher'

    "More autocomplete
    Plug 'tpope/vim-surround'

    " Respect Common Editorconfig stuff
    Plug 'editorconfig/editorconfig-vim'
    
    " Markdown
    "Plug 'tpope/vim-markdown'
    "Plug 'godlygeek/tabular'
    "Plug 'plasticboy/vim-markdown'
     Plug 'gabrielelana/vim-markdown'
    
    " Completion --> Obsoleted by YouCompleteMe
   " Plug 'othree/vim-autocomplpop'
   " Plug 'Shougo/neocomplete.vim'

    " Closers
    Plug 'Raimondi/delimitMate'

    " Padding
    Plug 'tpope/vim-sleuth'

    " Tmux Love
    Plug 'christoomey/vim-tmux-navigator'
    Plug 'benmills/vimux'
    " This in-case the status-bar gets too much..
    Plug 'edkolev/tmuxline.vim'

    " Prompt Love
    Plug 'edkolev/promptline.vim'

    " Ruby ending
    Plug 'tpope/vim-endwise'

    " Rails
    Plug 'scrooloose/vim-rails'

    " Pretty
    Plug 'ryanoasis/vim-devicons'
call plug#end()
call plug#helptags()
if !has('g:syntax_on')|syntax enable|endif
"filetype plugin indent on
"filetype indent on
" Behold paste indenting!
set pastetoggle=<F2>
set showmode
" setlocal spell spelllang=en_us
set grepprg=grep\ -nH\ $*
set showcmd        " display incomplete commands
"set autoread       " reload files (no local changes only)
set laststatus=2    " Always display the statusline in all windows
set showtabline=2   " Always display the tabline, even if there is only one tab
set noshowmode      " Hide the default mode text (e.g. -- INSERT -- below the statusline)
"set t_Co=256                 " force vim to use 256 colors
set number                  "Show line numbers              
set hidden                  "Buffers can exist in the background without being in a window.
"set nobackup       "no backup files
"set nowritebackup  "only in case you don't want a backup file while editing
"set noswapfile     "no swap files
" Too much ?
"autocmd TextChanged,TextChangedI <buffer> silent write
set backupdir=~/.vim/VimExtras/VimBackups//,/tmp
set directory=~/.vim/VimExtras/VimSwap//,/tmp
set showmatch      "automatically highlight matching braces/brackets/etc.
set history=1000         " remember more commands and search history
set undolevels=1000      " use many muchos levels of undo
autocmd GUIEnter * set visualbell t_vb=
"set visualbell           " don't beep
"set noerrorbells         " don't beep
" ================ Mappings ======================
" Probably NOT a good idea
"nnoremap <F2> :set invpaste paste?<CR>
nnoremap ; :
"Spacemacs Esc
inoremap <S-Space> <ESC>
"Fake Sudo
cmap w!! w !sudo tee % >/dev/null
" Allow j and k to be more normal.
nmap j gj
nmap k gk
" Use Q for formatting the current paragraph (or selection)
vmap Q gq
nmap Q gqap

" SET LEADER
let mapleader=","
let maplocalleader = "\\"
map <leader>n :new<cr>
map <leader>i I
map <leader><c-p> :CtrlPBookmarkDir<CR>
map <c-b> :CtrlPBuffer<CR>
map <c-h> :CtrlPMRUFiles<CR>
" ================ Indentation ======================

" set autoindent "Set in vim-sensible
"set smartindent       "Smart autoindenting on new line
set nowrap                      " don't wrap lines
set smarttab          "Respect space/tab settings
set shiftwidth=2
"set softtabstop=2
set tabstop=2
"set expandtab
"set backspace=indent,eol,start     "allow backspacing everything in insert mode, also in vim-sensible
set textwidth=70
set formatoptions+=t 
" ================ Folds ============================

set foldmethod=syntax   "fold based on indent
set foldnestmax=3       "deepest fold is 3 levels
set nofoldenable        "dont fold by default
set foldlevelstart=10   " fold very nested folds by default
set foldnestmax=10      " allow up to 10 nested folds
"nnoremap <silent> <Space> @=(foldlevel('.')?'za':"\<Space>")<CR>
"vnoremap <Space> zf

" ================ Searches ============================
set incsearch      " do incremental searching
set hlsearch        " Highlight searches by default
set ignorecase      " Ignore case when searching...
set smartcase       " ...unless we type a capital
map  / <Plug>(easymotion-sn)
omap / <Plug>(easymotion-tn)

" These `n` & `N` mappings are options. You do not have to map `n` & `N` to EasyMotion.
" Without these mappings, `n` & `N` works fine. (These mappings just provide
" different highlight method and have some other features )
map  n <Plug>(easymotion-next)
map  N <Plug>(easymotion-prev)


" Ctrl P Stuff
let g:ctrlp_user_command = 'ag %s -l --nocolor --hidden -g "" --ignore "\.git$\|\.hg$\|\.svn$"'
let g:ctrlp_map = '<c-p>'
let g:ctrlp_cmd = 'CtrlP'
let g:ctrlp_root_markers = ['.ctrlp','.latexmain','.agignore']
let g:ctrlp_working_path_mode = 0 
let g:ctrlp_use_caching = 1
let g:ctrlp_match_func = { 'match': 'pymatcher#PyMatch' }
" Use a .agignore instead for a per project thing. Also for a global thing.

 "let g:ctrlp_custom_ignore = {
 " \ 'dir':  '\v[\/]\.(git|hg|svn|cache|)$',
 " \ 'file': '\v\.(exe|so|dll|dll|log|lof|swp|out)$',
 " \ 'link': 'some_bad_symbolic_links',
 " \ }


" GUI STUFF
    if has("gui_running")
       " any code here affects gvim but not console vim
set background=dark
set guifont=CaskaydiaCove\ Nerd\ Font\ Regular\ 15
 " Don’t blink cursor in normal mode
set guicursor=n:blinkon0
" Better line-height
set linespace=8
let g:Powerline_symbols='fancy'
set encoding=utf-8
set termencoding=utf-8
set t_Co=256
    else
       " any code here affects console vim but not gvim
    endif 


" YouCompleteMe Stuff
"use omnicomplete whenever there's no completion engine in youcompleteme (for
"example, in the case of PHP)
set omnifunc=syntaxcomplete#Complete
" let g:ycm_key_invoke_completion = '<C-s>' " Ctrl-suggest - doesn't work
" because C-s freezes the command line
let g:ycm_key_invoke_completion = '<C-v>'
" let g:ycm_key_list_select_completion = ['<C-j>']
" let g:ycm_key_list_previous_completion = ['<C-k>']
let g:ycm_key_list_select_completion = ['<Tab>']
let g:ycm_key_list_previous_completion = ['<S-Tab>']

" If you want :UltiSnipsEdit to split your window.
let g:UltiSnipsEditSplit="vertical"



" Common Latex Stuff
let g:tex_flavor = 'latex'

" Vimtex
  let g:vimtex_view_general_viewer = 'zathura'
  let g:vimtex_view_method = 'zathura'
  let g:vimtex_latexmk_engine = '-pdfxe'
"  let g:vimtex_view_general_options = '--unique @pdf\#src:@line@tex'
"  let g:vimtex_view_general_options_latexmk = '--unique'
  let g:vimtex_latexmk_options= '--bibtex --silent -shell-escape -quiet --synctex=-1 -src-specials -pdfxe'
  let g:vimtex_quickfix_ignored_warnings = [
        \ 'Underfull',
        \ 'Overfull',
        \ 'specifier changed to',
        \ 'wrong kind of dash'
      \ ]
" let g:vimtex_complete_enabled
" let g:vimtex_complete_patterns.ref
" let g:vimtex_complete_patterns.bib
" let g:vimtex_complete_close_braces
" let g:vimtex_complete_recursive_bib

"YouCompleteMe
  if !exists('g:ycm_semantic_triggers')
    let g:ycm_semantic_triggers = {}
  endif
  let g:ycm_semantic_triggers.tex = [
        \ 're!\\[A-Za-z]*(ref|cite)[A-Za-z]*([^]]*])?{([^}]*, ?)*'
        \ ]



" Latex Box
"map <Leader> `
" let g:Tex_DefaultTargetFormat = 'pdf'  
" let g:Tex_BibtexFlavor = 'biber'
" let g:Tex_CompileRule_pdf = 'latexmk --bibtex --silent -quiet --synctex=-1 -src-specials -interaction=nonstopmode -pdf $*'
" let g:Tex_CompileRule_dvi = 'latexmk --bibtex --synctex=-1 -src-specials -interaction=nonstopmode -dvi $*'
" let g:Tex_ViewRule_dvi = 'okular $*.dvi &'
 "let g:Tex_ViewRule_pdf = 'okular $*.pdf &'
" let g:Tex_IgnoreLevel = 5
" map <Leader>lb :<C-U>exec '!biber '.Tex_GetMainFileName(':p:t:r')<CR>
 "let g:Tex_MultipleCompileFormats = 'pdf, dvi' Not needed cuz now its all latexmk

" Splitting Stuff

" Easy window navigation
" map <C-h> <C-w>h
" map <C-j> <C-w>j
" map <C-k> <C-w>k
" map <C-l> <C-w>l



" Vim Airline Stuff
" Use the airline thing anyway.
let g:miniBufExplAutoStart = 0
 let g:airline#extensions#tabline#enabled = 1
 let g:airline#extensions#tmuxline#enabled = 0 " Don't interefere with tmuxline
 let g:tmuxline_theme = 'powerline'
 set laststatus=2 		"Show statusbar always.
 let g:airline_powerline_fonts = 1
 let g:airline_theme='powerlineish'
 let g:airline#extensions#tabline#buffer_nr_show = 1
      nmap <leader>1 <Plug>AirlineSelectTab1
      nmap <leader>2 <Plug>AirlineSelectTab2
      nmap <leader>3 <Plug>AirlineSelectTab3
      nmap <leader>4 <Plug>AirlineSelectTab4
      nmap <leader>5 <Plug>AirlineSelectTab5
      nmap <leader>6 <Plug>AirlineSelectTab6
      nmap <leader>7 <Plug>AirlineSelectTab7
      nmap <leader>8 <Plug>AirlineSelectTab8
      nmap <leader>9 <Plug>AirlineSelectTab9
" Promptline Stuff
" sections (a, b, c, x, y, z, warn) are optional
let g:promptline_preset = {
        \'a' : [ promptline#slices#host({ 'only_if_ssh': 1 }) ],
        \'b' : [ promptline#slices#user(), promptline#slices#python_virtualenv() ],
        \'c' : [ promptline#slices#cwd() ],
        \'y' : [ promptline#slices#vcs_branch() ],
        \'warn' : [ promptline#slices#last_exit_code() ]}

" available slices:
"
" promptline#slices#cwd() - current dir, truncated to 3 dirs. To configure: promptline#slices#cwd({ 'dir_limit': 4 })
" promptline#slices#vcs_branch() - branch name only. By default, only git branch is enabled. Use promptline#slices#vcs_branch({ 'hg': 1, 'svn': 1, 'fossil': 1}) to enable check for svn, mercurial and fossil branches. Note that always checking if inside a branch slows down the prompt
" promptline#slices#last_exit_code() - display exit code of last command if not zero
" promptline#slices#jobs() - display number of shell jobs if more than zero
" promptline#slices#battery() - display battery percentage (on OSX and linux) only if below 10%. Configure the threshold with promptline#slices#battery({ 'threshold': 25 })
" promptline#slices#host() - current hostname.  To hide the hostname unless connected via SSH, use promptline#slices#host({ 'only_if_ssh': 1 })
" promptline#slices#user()
" promptline#slices#python_virtualenv() - display which virtual env is active (empty is none)
" promptline#slices#git_status() - count of commits ahead/behind upstream, count of modified/added/unmerged files, symbol for clean branch and symbol for existing untraced files
"
" any command can be used in a slice, for example to print the output of whoami in section 'b':
"       \'b' : [ '$(whoami)'],
"
" more than one slice can be placed in a section, e.g. print both host and user in section 'a':
"       \'a': [ promptline#slices#host(), promptline#slices#user() ],
"
" to disable powerline symbols
" `let g:promptline_powerline_symbols = 0`






" Automatically load .vimrc source when saved
autocmd BufWritePost .vimrc source ~/.vimrc


" Vim Markdown Stuff
let g:vim_markdown_math=1
let g:vim_markdown_frontmatter=1

" Vim Theme 
 colorscheme molokai


" More Stuff
" Vim jump to the last position when reopening a file
if has("autocmd")
 au BufReadPost * if line("'\"") > 0 && line("'\"") <= line("$")
 \| exe "normal! g'\"" | endif
endif






" nnoremap <C-S-u>  :GundoToggle<CR>
" Syntastic Defaults
"set statusline+=%#warningmsg#
"set statusline+=%{SyntasticStatuslineFlag()}
"set statusline+=%*

let g:syntastic_always_populate_loc_list = 1
let g:syntastic_auto_loc_list = 1
let g:syntastic_check_on_open = 1
let g:syntastic_check_on_wq = 0
let g:syntastic_tex_chktex_showmsgs = 0
let g:syntastic_tex_checkers=['chktex'] 

" NERDtree Key bound to Ctrl+n
map <C-n> :NERDTreeToggle<CR>

" NERDtree FileType Highlighting
function! NERDTreeHighlightFile(extension, fg, bg, guifg, guibg)
	 exec 'autocmd FileType nerdtree highlight ' . a:extension .' ctermbg='. a:bg .' ctermfg='. a:fg .' guibg='. a:guibg .' guifg='. a:guifg
	  exec 'autocmd FileType nerdtree syn match ' . a:extension .' #^\s\+.*'. a:extension .'$#'
  endfunction

  call NERDTreeHighlightFile('jade', 'green', 'none', 'green', '#151515')
  call NERDTreeHighlightFile('ini', 'yellow', 'none', 'yellow', '#151515')
  call NERDTreeHighlightFile('md', 'blue', 'none', '#3366FF', '#151515')
  call NERDTreeHighlightFile('yml', 'yellow', 'none', 'yellow', '#151515')
  call NERDTreeHighlightFile('config', 'yellow', 'none', 'yellow', '#151515')
  call NERDTreeHighlightFile('conf', 'yellow', 'none', 'yellow', '#151515')
  call NERDTreeHighlightFile('json', 'yellow', 'none', 'yellow', '#151515')
  call NERDTreeHighlightFile('html', 'yellow', 'none', 'yellow', '#151515')
  call NERDTreeHighlightFile('styl', 'cyan', 'none', 'cyan', '#151515')
  call NERDTreeHighlightFile('css', 'cyan', 'none', 'cyan', '#151515')
  call NERDTreeHighlightFile('coffee', 'Red', 'none', 'red', '#151515')
  call NERDTreeHighlightFile('js', 'Red', 'none', '#ffa500', '#151515')
  call NERDTreeHighlightFile('php', 'Magenta', 'none', '#ff00ff', '#151515')

" For the NERDtree-git-plugin
let g:NERDTreeGitStatusIndicatorMapCustom = {
    \ "Modified"  : "✹",
    \ "Staged"    : "✚",
    \ "Untracked" : "✭",
    \ "Renamed"   : "➜",
    \ "Unmerged"  : "═",
    \ "Deleted"   : "✖",
    \ "Dirty"     : "✗",
    \ "Clean"     : "✔︎",
    \ "Unknown"   : "?"
    \ }

" This allows the sensible settings to be overriden.
 runtime! plugin/sensible.vim

if exists("g:loaded_webdevicons")
    call webdevicons#refresh()
endif
