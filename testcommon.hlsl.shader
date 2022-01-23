static v2f_customrendertexture vertex_output_1;
#define txBuf _MainTex
#define txSize iChannelResolution[0].xy
#define mPtr _Mouse
            float Hashff(float p)
            {
                const float cHashM = 43758.54;
                return frac(sin(p)*cHashM);
            }

            static const float txRow = 32.;
            float4 Loadv4(int idVar)
            {
                float fi = float(idVar);
                return tex2D(txBuf, (float2(glsl_mod(fi, txRow), floor(fi/txRow))+0.5)/txSize);
            }

            void Savev4(int idVar, float4 val, inout float4 fCol, float2 fCoord)
            {
                float fi = float(idVar);
                float2 d = abs(fCoord-float2(glsl_mod(fi, txRow), floor(fi/txRow))-0.5);
                if (max(d.x, d.y)<0.5)
                    fCol = val;
                    
            }

            static const int nMolEdge = 20;
            static const int nMol = nMolEdge*nMolEdge;
            static float bFac;
            float4 Step(int mId)
            {
                float4 p, pp;
                float2 dr, f;
                float rr, rri, rri3, rCut, rrCut, bLen, dt;
                f = ((float2)0.);
                p = Loadv4(mId);
                rCut = pow(2., 1./6.);
                rrCut = rCut*rCut;
                for (int n = 0;n<nMol; n++)
                {
                    pp = Loadv4(n);
                    dr = p.xy-pp.xy;
                    rr = dot(dr, dr);
                    if (n!=mId&&rr<rrCut)
                    {
                        rri = 1./rr;
                        rri3 = rri*rri*rri;
                        f += 48.*rri3*(rri3-0.5)*rri*dr;
                    }
                    
                }
                bLen = bFac*float(nMolEdge);
                dr = 0.5*(bLen+rCut)-abs(p.xy);
                if (dr.x<rCut)
                {
                    if (p.x>0.)
                        dr.x = -dr.x;
                        
                    rri = 1./(dr.x*dr.x);
                    rri3 = rri*rri*rri;
                    f.x += 48.*rri3*(rri3-0.5)*rri*dr.x;
                }
                
                if (dr.y<rCut)
                {
                    if (p.y>0.)
                        dr.y = -dr.y;
                        
                    rri = 1./(dr.y*dr.y);
                    rri3 = rri*rri*rri;
                    f.y += 48.*rri3*(rri3-0.5)*rri*dr.y;
                }
                
                dt = 0.005;
                p.zw += dt*f;
                p.xy += dt*p.zw;
                return p;
            }

            float4 Init(int mId)
            {
                float4 p;
                float x, y, t, vel;
                const float pi = 3.14159;
                y = float(mId/nMolEdge);
                x = float(mId)-float(nMolEdge)*y;
                t = 0.25*(2.*glsl_mod(y, 2.)-1.);
                p.xy = float2(x+t, y)-0.5*float(nMolEdge-1);
                t = 2.*pi*Hashff(float(mId));
                vel = 3.;
                p.zw = vel*float2(cos(t), sin(t));
                return p;
            }

            float4 frag (v2f_customrendertexture __vertex_output) : SV_Target
            {
                vertex_output_1 = __vertex_output;
                float4 fragColor = 0;
                float2 fragCoord = vertex_output_1.uv * _Resolution;
                float4 stDat, p;
                int mId;
                float2 kv = floor(fragCoord);
                mId = int(kv.x+txRow*kv.y);
                if (kv.x>=txRow||mId>nMol)
                    discard;
                    
                if (iFrame<=5)
                {
                    bFac = 1.1;
                    stDat = float4(0., bFac, 0., 0.);
                    if (mId<nMol)
                        p = Init(mId);
                        
                }
                else 
                {
                    stDat = Loadv4(nMol);
                    ++stDat.x;
                    bFac = stDat.y;
                    if (mId<nMol)
                        p = Step(mId);
                        
                    if (mPtr.z>0.&&stDat.x>50.)
                    {
                        stDat.x = 0.;
                        p = Init(mId);
                    }
                    
                }
                Savev4(mId, mId<nMol ? p : stDat, fragColor, fragCoord);
                                return fragColor;
            }
static v2f_customrendertexture vertex_output_2;
#define txBuf _MainTex_1
#define txSize iChannelResolution[0].xy
#define mPtr _Mouse
            float Hashff(float p)
            {
                const float cHashM = 43758.54;
                return frac(sin(p)*cHashM);
            }

            static const float txRow = 32.;
            float4 Loadv4(int idVar)
            {
                float fi = float(idVar);
                return tex2D(txBuf, (float2(glsl_mod(fi, txRow), floor(fi/txRow))+0.5)/txSize);
            }

            void Savev4(int idVar, float4 val, inout float4 fCol, float2 fCoord)
            {
                float fi = float(idVar);
                float2 d = abs(fCoord-float2(glsl_mod(fi, txRow), floor(fi/txRow))-0.5);
                if (max(d.x, d.y)<0.5)
                    fCol = val;
                    
            }

            static const int nMolEdge = 20;
            static const int nMol = nMolEdge*nMolEdge;
            static float bFac;
            float4 Step(int mId)
            {
                float4 p, pp;
                float2 dr, f;
                float rr, rri, rri3, rCut, rrCut, bLen, dt;
                f = ((float2)0.);
                p = Loadv4(mId);
                rCut = pow(2., 1./6.);
                rrCut = rCut*rCut;
                for (int n = 0;n<nMol; n++)
                {
                    pp = Loadv4(n);
                    dr = p.xy-pp.xy;
                    rr = dot(dr, dr);
                    if (n!=mId&&rr<rrCut)
                    {
                        rri = 1./rr;
                        rri3 = rri*rri*rri;
                        f += 48.*rri3*(rri3-0.5)*rri*dr;
                    }
                    
                }
                bLen = bFac*float(nMolEdge);
                dr = 0.5*(bLen+rCut)-abs(p.xy);
                if (dr.x<rCut)
                {
                    if (p.x>0.)
                        dr.x = -dr.x;
                        
                    rri = 1./(dr.x*dr.x);
                    rri3 = rri*rri*rri;
                    f.x += 48.*rri3*(rri3-0.5)*rri*dr.x;
                }
                
                if (dr.y<rCut)
                {
                    if (p.y>0.)
                        dr.y = -dr.y;
                        
                    rri = 1./(dr.y*dr.y);
                    rri3 = rri*rri*rri;
                    f.y += 48.*rri3*(rri3-0.5)*rri*dr.y;
                }
                
                dt = 0.005;
                p.zw += dt*f;
                p.xy += dt*p.zw;
                return p;
            }

            float4 Init(int mId)
            {
                float4 p;
                float x, y, t, vel;
                const float pi = 3.14159;
                y = float(mId/nMolEdge);
                x = float(mId)-float(nMolEdge)*y;
                t = 0.25*(2.*glsl_mod(y, 2.)-1.);
                p.xy = float2(x+t, y)-0.5*float(nMolEdge-1);
                t = 2.*pi*Hashff(float(mId));
                vel = 3.;
                p.zw = vel*float2(cos(t), sin(t));
                return p;
            }

            float4 frag (v2f_customrendertexture __vertex_output) : SV_Target
            {
                vertex_output_2 = __vertex_output;
                float4 fragColor = 0;
                float2 fragCoord = vertex_output_2.uv * _Resolution;
                float4 stDat, p;
                int mId;
                float2 kv = floor(fragCoord);
                mId = int(kv.x+txRow*kv.y);
                if (kv.x>=txRow||mId>nMol)
                    discard;
                    
                if (iFrame<=5)
                {
                    bFac = 1.1;
                    stDat = float4(0., bFac, 0., 0.);
                    if (mId<nMol)
                        p = Init(mId);
                        
                }
                else 
                {
                    stDat = Loadv4(nMol);
                    ++stDat.x;
                    bFac = stDat.y;
                    if (mId<nMol)
                        p = Step(mId);
                        
                    if (mPtr.z>0.&&stDat.x>50.)
                    {
                        stDat.x = 0.;
                        p = Init(mId);
                    }
                    
                }
                Savev4(mId, mId<nMol ? p : stDat, fragColor, fragCoord);
                                return fragColor;
            }
static v2f_customrendertexture vertex_output_3;
#define txBuf _MainTex_2
#define txSize iChannelResolution[0].xy
#define mPtr _Mouse
            float Hashff(float p)
            {
                const float cHashM = 43758.54;
                return frac(sin(p)*cHashM);
            }

            static const float txRow = 32.;
            float4 Loadv4(int idVar)
            {
                float fi = float(idVar);
                return tex2D(txBuf, (float2(glsl_mod(fi, txRow), floor(fi/txRow))+0.5)/txSize);
            }

            void Savev4(int idVar, float4 val, inout float4 fCol, float2 fCoord)
            {
                float fi = float(idVar);
                float2 d = abs(fCoord-float2(glsl_mod(fi, txRow), floor(fi/txRow))-0.5);
                if (max(d.x, d.y)<0.5)
                    fCol = val;
                    
            }

            static const int nMolEdge = 20;
            static const int nMol = nMolEdge*nMolEdge;
            static float bFac;
            float4 Step(int mId)
            {
                float4 p, pp;
                float2 dr, f;
                float rr, rri, rri3, rCut, rrCut, bLen, dt;
                f = ((float2)0.);
                p = Loadv4(mId);
                rCut = pow(2., 1./6.);
                rrCut = rCut*rCut;
                for (int n = 0;n<nMol; n++)
                {
                    pp = Loadv4(n);
                    dr = p.xy-pp.xy;
                    rr = dot(dr, dr);
                    if (n!=mId&&rr<rrCut)
                    {
                        rri = 1./rr;
                        rri3 = rri*rri*rri;
                        f += 48.*rri3*(rri3-0.5)*rri*dr;
                    }
                    
                }
                bLen = bFac*float(nMolEdge);
                dr = 0.5*(bLen+rCut)-abs(p.xy);
                if (dr.x<rCut)
                {
                    if (p.x>0.)
                        dr.x = -dr.x;
                        
                    rri = 1./(dr.x*dr.x);
                    rri3 = rri*rri*rri;
                    f.x += 48.*rri3*(rri3-0.5)*rri*dr.x;
                }
                
                if (dr.y<rCut)
                {
                    if (p.y>0.)
                        dr.y = -dr.y;
                        
                    rri = 1./(dr.y*dr.y);
                    rri3 = rri*rri*rri;
                    f.y += 48.*rri3*(rri3-0.5)*rri*dr.y;
                }
                
                dt = 0.005;
                p.zw += dt*f;
                p.xy += dt*p.zw;
                return p;
            }

            float4 Init(int mId)
            {
                float4 p;
                float x, y, t, vel;
                const float pi = 3.14159;
                y = float(mId/nMolEdge);
                x = float(mId)-float(nMolEdge)*y;
                t = 0.25*(2.*glsl_mod(y, 2.)-1.);
                p.xy = float2(x+t, y)-0.5*float(nMolEdge-1);
                t = 2.*pi*Hashff(float(mId));
                vel = 3.;
                p.zw = vel*float2(cos(t), sin(t));
                return p;
            }

            float4 frag (v2f_customrendertexture __vertex_output) : SV_Target
            {
                vertex_output_3 = __vertex_output;
                float4 fragColor = 0;
                float2 fragCoord = vertex_output_3.uv * _Resolution;
                float4 stDat, p;
                int mId;
                float2 kv = floor(fragCoord);
                mId = int(kv.x+txRow*kv.y);
                if (kv.x>=txRow||mId>nMol)
                    discard;
                    
                if (iFrame<=5)
                {
                    bFac = 1.1;
                    stDat = float4(0., bFac, 0., 0.);
                    if (mId<nMol)
                        p = Init(mId);
                        
                }
                else 
                {
                    stDat = Loadv4(nMol);
                    ++stDat.x;
                    bFac = stDat.y;
                    if (mId<nMol)
                        p = Step(mId);
                        
                    if (mPtr.z>0.&&stDat.x>50.)
                    {
                        stDat.x = 0.;
                        p = Init(mId);
                    }
                    
                }
                Savev4(mId, mId<nMol ? p : stDat, fragColor, fragCoord);
                                return fragColor;
            }
