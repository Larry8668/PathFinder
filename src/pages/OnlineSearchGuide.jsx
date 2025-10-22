import { Link } from "react-router-dom"
import snapshot3 from '../assets/snapshort3.png'


function OnlineSearchGuide(){
    return(
        <div className="flex flex-col justify-center gap-5 p-5 h-full bg-[#3D3C3C] font-sans text-white">
            <div className="felx flex-row justify-left text-left text-4xl ">Online Search:</div>
            <div className="text-left backdrop-blur-sm">
                Quick-access web search allows users to instantly search the internet directly from the app. With a streamlined interface, it supports rapid queries, displays results efficiently, and saves frequently used searches, enabling fast, convenient, and productive online information retrieval.                </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshot3} alt='snapshot1'
                className="flex justify-center items-center rounded-t-xl "/>
            </div>
            <Link to='/OpenFileGuide'><button className="relative bottom-3 left-170 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button></Link>

        </div>
    )
} 
export default OnlineSearchGuide