//! Responsabilite : les angles arrondis de la page. Une vue web est un rectangle natif que Chromium ne sait
//! pas rogner. On pose donc dans chaque page quatre masques d'angle de la teinte de la fenetre ; la teinte
//! se change a chaud (bascule clair/sombre) sans recharger la page.

/// Rayon des angles, en pixels CSS.
const RADIUS: u32 = 12;

/// Actif par defaut ; `ECHO_ROUND=0` le coupe, pour comparer.
pub fn enabled() -> bool {
    std::env::var("ECHO_ROUND").map_or(true, |v| v != "0")
}

/// `#rrggbb` depuis la teinte ARGB de la fenetre.
pub fn css_color(argb: u32) -> String {
    format!("#{:06x}", argb & 0x00FF_FFFF)
}

/// Le code a executer au debut du chargement d'une page : pose les masques, et expose `window.__echoCorners(couleur)`
/// pour les recolorer.
pub fn install_script(argb: u32) -> String {
    format!(
        r#"(()=>{{
const R={RADIUS};let color='{color}';
const corners=[['top','left','100% 100%'],['top','right','0% 100%'],['bottom','left','100% 0%'],['bottom','right','0% 0%']];
let host=null;
const paint=()=>{{if(!host)return;for(const el of host.children){{el.style.background=
`radial-gradient(circle at ${{el.dataset.at}}, transparent ${{R-0.5}}px, ${{color}} ${{R+0.5}}px)`;}}}};
const build=()=>{{host=document.createElement('div');host.id='__echo-corners';
host.style.cssText='position:fixed;inset:0;pointer-events:none;z-index:2147483647;contain:strict';
for(const [v,h,at] of corners){{const c=document.createElement('div');c.dataset.at=at;
c.style.cssText=`position:absolute;${{v}}:0;${{h}}:0;width:${{R}}px;height:${{R}}px`;host.appendChild(c);}}
paint();return host;}};
const attach=()=>{{const root=document.documentElement;if(!root)return;if(!host)build();if(!host.isConnected)root.appendChild(host);}};
window.__echoCorners=(c)=>{{color=c;paint();}};
attach();
addEventListener('DOMContentLoaded',attach);
addEventListener('fullscreenchange',()=>{{if(host)host.style.display=document.fullscreenElement?'none':'';}});
new MutationObserver(()=>{{if(host&&!host.isConnected)attach();}}).observe(document,{{childList:true,subtree:false}});
}})()"#,
        color = css_color(argb),
    )
}

/// Recolore les masques deja poses.
pub fn recolor_script(argb: u32) -> String {
    format!("window.__echoCorners&&window.__echoCorners('{}')", css_color(argb))
}
