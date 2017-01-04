package {'graphviz': ensure => present }
#package {'texlive-bundledoc': ensure => present }
#package {'texlive-scheme-full': ensure => present }
package {'texlive-scheme-basic': ensure => present }
package {'perl-Text-Template': ensure => present }

# For plantuml.
package {'java-1.8.0-openjdk-headless': ensure => present }

# TODO rackdiag + nwdiag
