\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{BA}[2020/01/08 Class for my Bachelor thesis]

% use article as the base class
\LoadClass[12pt,a4paper]{article}

% load packages
\RequirePackage{etex}
\RequirePackage[utf8]{inputenc} % utf8 Support
%\RequirePackage[ngerman]{babel}
\RequirePackage[table]{xcolor} % more colors
\RequirePackage{fancyhdr} % custome page style
\RequirePackage[left=2cm,right=2cm,top=2cm,bottom=2cm,includeheadfoot,headheight=35pt]{geometry} % borders
\RequirePackage{ifthen}
\RequirePackage{graphicx}
\RequirePackage{csquotes}
\RequirePackage{setspace}
\RequirePackage{footnote}
\RequirePackage{hyperref}
\RequirePackage{tabularx}
\RequirePackage{dirtytalk}
\RequirePackage{amsmath}
\RequirePackage{subcaption}
\RequirePackage{listings}
\RequirePackage{pdfpages}

\setlength{\marginparwidth}{2cm}

% References
\RequirePackage[
    backend=biber,
    bibencoding=utf8,
    style=ieee
]{biblatex}
\ExecuteBibliographyOptions{sorting=nty}

% Prevent Footnotes from breaking over pages
\interfootnotelinepenalty=1000000

\newcommand{\todo}[1]{{\color{red}TODO: \textbf{#1}}}
\newcommand{\code}[1]{\texttt{#1}}
\newcommand{\marker}[1]{\colorbox{yellow!30}{#1}}
\newcommand{\citationneeded}{\textsuperscript{\marker{[citation needed]}}}
\newcommand{\addcitation}[1]{\textsuperscript{\marker{[add citation]}}}

\newcommand{\srcSelfFigure}{\textit{(Eigene Darstellung)}}
\newcommand{\srcSelfFigureCaption}[1]{\caption[#1]{#1 \srcSelfFigure{}}}
\newcommand{\srcSelfList}{\textit{(Eigene Auflistung)}}
\newcommand{\srcSelfListCaption}[1]{\caption[#1]{#1 \srcSelfList{}}}

\lstdefinestyle{numbered}{
    breakatwhitespace=false,
    breaklines=true,
    keepspaces=true,
    numbers=left,
    numbersep=5pt,                  
    showspaces=false,
    showstringspaces=false,
    showtabs=false,
    tabsize=2
}

%%%%%%%%%%%%%%%
%% Numbering %%
%%%%%%%%%%%%%%%

\newcommand{\setupnumbering}{
    \newcounter{numberingtype}
    \setcounter{numberingtype}{1}
    
    \newcounter{pageroman}
    \setcounter{pageroman}{1}
    
    \newcounter{pagearabic}
    \setcounter{pagearabic}{1}
    
    \newcommand{\resetnumbering}{
        \ifthenelse{\value{numberingtype} = 1}{
            \setcounter{pagearabic}{\value{page}}
        }{
            \ifthenelse{\value{numberingtype} = 2}{
                \setcounter{pageroman}{\value{page}}
            }{
                
            }
        }
    }
    
    \newcommand{\disablenumbering}{
        \resetnumbering
        \setcounter{numberingtype}{0}
    	\pagenumbering{gobble}}
    
    \newcommand{\arabicnumbering}{
        \resetnumbering
        \setcounter{numberingtype}{1}
    	\pagenumbering{arabic}
    	\setcounter{page}{\value{pagearabic}}}
    
    \newcommand{\Romannumbering}{
        \resetnumbering
        \setcounter{numberingtype}{2}
    	\pagenumbering{Roman}
    	\setcounter{page}{\value{pageroman}}}
    
    \newenvironment{titlematter}{\disablenumbering}{\newpage}
    \newenvironment{frontmatter}{\Romannumbering}{\newpage}
    \newenvironment{mainmatter}{\arabicnumbering}{\newpage}
    \newenvironment{backmatter}{
        \Romannumbering
        \setcounter{section}{0}
        \renewcommand{\thesection}{\Roman{section}}
    }{}
}

\newcommand{\setupnordakademie}{
    \setupnumbering
    
    % Linespacing 1.5
    \onehalfspacing
    
    \definecolor{nordakademie-blue}{RGB}{002,034,094}
    % \pagestyle{fancy}
    % \fancyhf{}
    % \fancyhead[R]{\includegraphics[height=20pt]{image/nordakademie_logo_small.jpg}\vspace*{2pt}}
    % \fancyfoot[C]{\thepage}
    \pagestyle{fancy}
    \lhead{\textcolor{nordakademie-blue}{\uppercase{Bachelor thesis}}}
    \rhead{\textcolor{nordakademie-blue}{\includegraphics[height=0.85cm,keepaspectratio]{image/nordakademie_logo_large.jpg}}}
    \cfoot{\thepage}
    
    \setcounter{tocdepth}{2}
    \newcommand{\inhaltsverzeichnis}{
    	\newpage
    	\hypertarget{toc}
    	\phantomsection
    	\tableofcontents
    	\clearpage
    }
    
    \newcommand{\abbildungsverzeichnis}{
    	% \newpage
    	\phantomsection
    	\addcontentsline{toc}{section}{\footnotesize{List of Figures}}
    	\listoffigures
    }
    
    \newcommand{\tabellenverzeichnis}{
    	% \newpage
    	\phantomsection
    	\addcontentsline{toc}{section}{\footnotesize{List of Tables}}
    	\listoftables
    }
    
    \newcommand{\abkuerzungsverzeichnis}{
    	% \newpage
    	\sectionX{Abkürzungsverzeichnis}
    	\input{content/acronyms.tex}
    }
    
    \newcommand{\printreferences}{
    	\newpage	
    	\phantomsection
    	\addcontentsline{toc}{section}{References}
    	\printbibliography
    }
    
    \newcommand{\createtitle}{
        \begin{titlepage}
        	\centering
        	\includegraphics[width=0.5\textwidth]{image/nordakademie_logo_large.jpg}\par\vspace{1cm}
        % 	{\scshape\LARGE Nordakademie \par}
        	\vspace{1cm}
        	{\scshape\Large Bachelor Thesis\par}
        	\vspace{1.5cm}
        	{\huge\bfseries Automating decision making for efficient CI pipeline artifact storage under disk space constraints\par}
        	\vspace{2cm}
        	{\Large\itshape Til Blechschmidt\par}
        	\vfill
        	supervised by\par
        	Dr.~Mario \textsc{Arkenberg}
        
        	\vfill
        
        	{\large \today\par}
        \end{titlepage}
    }
}

\newenvironment{Figure}
  {\par\bigskip\bigskip\noindent\minipage{\linewidth}\centering\captionsetup{type=figure}}
  {\endminipage\par\bigskip\bigskip}

\newcommand{\colfig}[3]{
    \begin{Figure}
        \includegraphics[width=\linewidth]{#1}
        \captionof{figure}[#2]{#2 [A]}
        \label{#3}
    \end{Figure}
}

\DeclareOption{nordakademie}{\setupnordakademie}
\DeclareOption{transferleistung}{\setuptransferleistung}
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{article}}
\ProcessOptions\relax